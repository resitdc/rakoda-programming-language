import 'dart:convert';
import 'dart:io';
import 'dart:collection';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import 'package:flutter_inappwebview/flutter_inappwebview.dart';
import 'package:webview_windows/webview_windows.dart' as win_web;
import 'package:uuid/uuid.dart';
import 'dart:async';
import 'devtools_panel.dart';

class BrowserWorkspace extends StatefulWidget {
  final String? initialUrl;
  const BrowserWorkspace({super.key, this.initialUrl});

  @override
  State<BrowserWorkspace> createState() => _BrowserWorkspaceState();
}

class _BrowserWorkspaceState extends State<BrowserWorkspace> {
  InAppWebViewController? _controller;
  late final InAppWebViewSettings _settings = InAppWebViewSettings(
    isInspectable: true,
    javaScriptEnabled: true,
    transparentBackground: false, // Transparent background can cause texture detachment
    useHybridComposition: false, // FORCE Virtual Display mode to bypass SurfaceAnimationManager crashes completely
  );
  final _windowsController = win_web.WebviewController();
  bool _isWindowsWebviewInitialized = false;
  final Map<String, Completer<dynamic>> _jsCallbacks = {};
  late final TextEditingController _urlController = TextEditingController(
    text: widget.initialUrl ?? 'https://flutter.dev',
  );
  bool _isLoading = true;
  double _progress = 0;

  // DevTools states
  List<String> _consoleLogs = [];
  String _pageSource = '';
  List<Map<String, dynamic>> _networkRequests = [];
  Map<String, String> _cookies = {};
  Map<String, String> _localStorage = {};
  bool _isDevToolsMinimized = true;

@override
  void dispose() {
    try {
      if (Platform.isWindows) {
        _windowsController.dispose();
      } else {
        
      }
    } catch (_) {}
    super.dispose();
  }

  @override
  void initState() {
    super.initState();

    if (Platform.isWindows) {
      _initWindowsWebview();
      return;
    }

    // Controller is initialized in onWebViewCreated for InAppWebView
  }

  Future<void> _extractDevToolsData() async {
    try {
      // Inject console override to capture logs
      await _runJavascript('''
        if (!window._consoleOverridden) {
          window._consoleOverridden = true;
          function sendLog(msg) {
             if (window.ConsoleChannel) {
                ConsoleChannel.postMessage(msg);
             } else if (window.chrome && window.chrome.webview) {
                window.chrome.webview.postMessage({ type: 'console', message: msg });
             }
          }
          const oldLog = console.log;
          const oldError = console.error;
          const oldWarn = console.warn;
          console.log = function(...args) {
            sendLog('[LOG] ' + args.join(' '));
            oldLog.apply(console, args);
          };
          console.error = function(...args) {
            sendLog('[ERROR] ' + args.join(' '));
            oldError.apply(console, args);
          };
          console.warn = function(...args) {
            sendLog('[WARN] ' + args.join(' '));
            oldWarn.apply(console, args);
          };
        }
      ''');

      // Inject network interceptor
      await _runJavascript('''
        if (!window._networkOverridden) {
          window._networkOverridden = true;

          // 1. Intercept Fetch
          function sendNet(obj) {
             if (window.NetworkChannel) {
                if (window.NetworkChannel) {
                NetworkChannel.postMessage(JSON.stringify(obj));
              } else if (window.chrome && window.chrome.webview) {
                obj.type = 'network';
                window.chrome.webview.postMessage(obj);
              }
             } else if (window.chrome && window.chrome.webview) {
                obj.type = 'network';
                window.chrome.webview.postMessage(obj);
             }
          }
          const originalFetch = window.fetch;
          window.fetch = async function(...args) {
            const url = args[0];
            const options = args[1] || {};
            const method = options.method || 'GET';
            const payload = options.body ? options.body.toString() : '';

            sendNet({
              url: url,
              method: method,
              status: 'Pending',
              payload: payload,
              response: '',
              contentType: 'application/json'
            });

            try {
              const response = await originalFetch(...args);
              const clone = response.clone();
              let responseText = '';
              try {
                responseText = await clone.text();
              } catch (_) {}
              
              sendNet({
                url: url,
                method: method,
                status: response.status + ' ' + response.statusText,
                payload: payload,
                response: responseText,
                contentType: response.headers.get('content-type') || 'application/json'
              });
              return response;
            } catch (err) {
              sendNet({
                url: url,
                method: method,
                status: 'Failed',
                payload: payload,
                response: err.toString(),
                contentType: 'text/plain'
              });
              throw err;
            }
          };

          // 2. Intercept XHR (XMLHttpRequest)
          const origOpen = XMLHttpRequest.prototype.open;
          const origSend = XMLHttpRequest.prototype.send;

          XMLHttpRequest.prototype.open = function(method, url, ...args) {
            this._url = url;
            this._method = method;
            return origOpen.apply(this, [method, url, ...args]);
          };

          XMLHttpRequest.prototype.send = function(body) {
            const xhr = this;
            const url = xhr._url;
            const method = xhr._method || 'GET';
            const payload = body ? body.toString() : '';

            sendNet({
              url: url,
              method: method,
              status: 'Pending',
              payload: payload,
              response: '',
              contentType: 'text/plain'
            });

            xhr.addEventListener('load', function() {
              sendNet({
                url: url,
                method: method,
                status: xhr.status + ' ' + xhr.statusText,
                payload: payload,
                response: xhr.responseText,
                contentType: xhr.getResponseHeader('content-type') || 'text/plain'
              });
            });

            xhr.addEventListener('error', function() {
              sendNet({
                url: url,
                method: method,
                status: 'Failed',
                payload: payload,
                response: 'Network Error',
                contentType: 'text/plain'
              });
            });

            return origSend.apply(this, arguments);
          };
        }
      ''');

      // 3. Inject Resource Timing collector for CSS, JS, Images, Media
      await _runJavascript('''
        (function() {
          const resources = performance.getEntriesByType('resource');
          for (const res of resources) {
            if (res.initiatorType !== 'xmlhttprequest' && res.initiatorType !== 'fetch') {
              let cType = 'text/plain';
              if (res.initiatorType === 'css') cType = 'text/css';
              else if (res.initiatorType === 'img') cType = 'image/png';
              else if (res.initiatorType === 'script') cType = 'text/javascript';

              sendNet({
                url: res.name,
                method: 'GET',
                status: '200 OK',
                payload: '',
                response: '[Resource Loaded from Cache/Network]',
                contentType: cType
              });
            }
          }
        })();
      ''');

      // Get page source (Truncate to prevent WebView bridge crash on massive pages like WP Admin)
      final html = await _evaluateJavascript(
        'document.documentElement.outerHTML.substring(0, 150000)',
      );
      String htmlStr = html.toString();
      if (htmlStr.startsWith('"') && htmlStr.endsWith('"')) {
        try {
          htmlStr = jsonDecode(htmlStr) as String;
        } catch (_) {}
      }
      setState(() {
        _pageSource = htmlStr;
      });

      // Get cookies
      final cookiesObj = await _evaluateJavascript(
        'document.cookie',
      );
      String cookiesStr = cookiesObj.toString();
      if (cookiesStr.startsWith('"') && cookiesStr.endsWith('"')) {
        try {
          cookiesStr = jsonDecode(cookiesStr) as String;
        } catch (_) {}
      }
      final Map<String, String> parsedCookies = {};
      if (cookiesStr.isNotEmpty && cookiesStr != '""') {
        final parts = cookiesStr.split(';');
        for (var part in parts) {
          if (part.contains('=')) {
            final idx = part.indexOf('=');
            final k = part.substring(0, idx).trim();
            final v = part.substring(idx + 1).trim();
            if (k.isNotEmpty) {
              parsedCookies[k] = v;
            }
          }
        }
      }
      setState(() {
        _cookies = parsedCookies;
      });

      // Get local storage
      final lsObj = await _evaluateJavascript(
        'JSON.stringify(localStorage)',
      );
      String lsStr = lsObj.toString();
      if (lsStr.startsWith('"') && lsStr.endsWith('"')) {
        try {
          lsStr = jsonDecode(lsStr) as String;
        } catch (_) {}
      }
      final Map<String, String> parsedLs = {};
      if (lsStr.isNotEmpty && lsStr != '{}') {
        try {
          final Map<String, dynamic> rawLs = jsonDecode(lsStr);
          rawLs.forEach((k, v) {
            parsedLs[k] = v.toString();
          });
        } catch (e) {
          debugPrint('Failed to parse localStorage JSON: $e');
        }
      }
      setState(() {
        _localStorage = parsedLs;
      });
    } catch (e) {
      if (e.toString().contains('FWFEvaluateJavaScriptError') || e.toString().contains('Failed evaluating JavaScript')) {
        // Abaikan error ini. Biasanya terjadi saat Hot Restart ketika context WebView terputus.
      } else {
        debugPrint('Failed to extract devtools data: $e');
      }
    }
  }

  void _loadUrl(String url) {
    url = url.trim();
    if (url.isEmpty) return;
    if (url == 'rpl://browser') {
      if (Platform.isWindows) {
        if (_isWindowsWebviewInitialized) _windowsController.loadStringContent(_getDefaultHtml());
      } else {
        _controller?.loadData(data: _getDefaultHtml());
      }
      FocusScope.of(context).unfocus();
      return;
    }

    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      if (url.startsWith('localhost') || url.startsWith('127.0.0.1') || url.startsWith('0.0.0.0')) {
        url = 'http://$url';
      } else if (url.contains('.') && !url.contains(' ')) {
        url = 'https://$url';
      } else {
        url = 'https://www.google.com/search?q=${Uri.encodeComponent(url)}';
      }
    }
    
    if (Platform.isWindows) {
      if (_isWindowsWebviewInitialized) _windowsController.loadUrl(url);
    } else {
      _controller?.loadUrl(urlRequest: URLRequest(url: WebUri(url)));
    }
    FocusScope.of(context).unfocus();
  }

  String _getDefaultHtml() {
    return '''
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>RPL Browser</title>
  <style>
    body {
      background-color: #1e1e1e;
      color: #ffffff;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      margin: 0;
      overflow: hidden;
    }
    .container {
      text-align: center;
      animation: fadeIn 0.8s ease-out;
    }
    @keyframes fadeIn {
      from { opacity: 0; transform: translateY(15px); }
      to { opacity: 1; transform: translateY(0); }
    }
    .logo-container {
      margin-bottom: 24px;
      display: flex;
      flex-direction: column;
      align-items: center;
    }
    .rakoda-logo {
      width: 50px;
      height: auto;
      margin-bottom: 26px;
      filter: drop-shadow(0 0 12px rgba(37, 104, 231, 0.5));
      animation: logoPulse 2s infinite alternate;
    }
    @keyframes logoPulse {
      from { transform: scale(1); filter: drop-shadow(0 0 10px rgba(37, 104, 231, 0.4)); }
      to { transform: scale(1.05); filter: drop-shadow(0 0 18px rgba(37, 104, 231, 0.7)); }
    }
    .logo-text {
      font-size: 28px;
      font-weight: 900;
      letter-spacing: 6px;
      color: #2568E7;
      text-shadow: 0 0 20px rgba(0, 122, 204, 0.4);
      margin: 0;
      background: linear-gradient(135deg, #2568e7, #00bfff);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    .sub-logo {
      font-size: 13px;
      color: #858585;
      letter-spacing: 5px;
      text-transform: uppercase;
      margin-top: 8px;
      font-weight: 600;
    }
    .title {
      font-size: 18px;
      color: #cccccc;
      margin-bottom: 36px;
      font-weight: 300;
    }
    .search-box {
      display: flex;
      width: 85%;
      max-width: 520px;
      margin: 0 auto;
      background-color: #252526;
      border: 1px solid #3c3c3c;
      border-radius: 28px;
      padding: 10px 20px;
      box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
      transition: all 0.3s;
    }
    .search-box:hover {
      border-color: #555555;
      box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    }
    .search-box:focus-within {
      border-color: #2568E7;
      box-shadow: 0 8px 24px rgba(0, 122, 204, 0.2);
    }
    .search-input {
      flex: 1;
      background: none;
      border: none;
      color: #ffffff;
      font-size: 15px;
      outline: none;
    }
  </style>
</head>
<body>
  <div class="container">
    <div class="logo-container">
      <svg class="rakoda-logo" viewBox="0 0 344 464" fill="none" xmlns="http://www.w3.org/2000/svg">
        <rect width="104" height="104" rx="52" fill="#2568E7"/>
        <rect width="104" height="104" fill="#2568E7"/>
        <rect y="120" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect y="120" width="104" height="104" fill="#2568E7"/>
        <rect y="240" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect y="240" width="104" height="104" fill="#2568E7"/>
        <rect x="120" y="240" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect x="120" y="240" width="104" height="104" fill="#2568E7"/>
        <rect x="120" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect x="120" width="104" height="104" fill="#2568E7"/>
        <rect x="240" y="360" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect x="240" y="360" width="104" height="104" fill="#2568E7"/>
        <rect x="240" y="120" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect x="240" y="120" width="104" height="104" fill="#2568E7"/>
        <rect y="360" width="104" height="104" rx="52" fill="#2568E7"/>
        <rect y="360" width="104" height="104" fill="#2568E7"/>
      </svg>
      <h1 class="logo-text">RPL STUDIO</h1>
    </div>
    <form class="search-box" action="https://www.google.com/search" method="get">
      <input class="search-input" type="text" name="q" placeholder="Cari di Google atau ketik URL...." required autocomplete="off">
    </form>
  </div>
</body>
</html>
''';
  }


  Future<void> _initWindowsWebview() async {
    try {
      await _windowsController.initialize();
      _windowsController.url.listen((url) {
        if (!mounted) return;
        setState(() {
          _urlController.text = (url.startsWith('data:text/html') || url == 'about:blank') ? 'rpl://browser' : url;
          _isLoading = true;
          _consoleLogs.clear();
          _networkRequests.add({
            'url': _urlController.text,
            'method': 'GET',
            'status': 'Pending',
            'time': DateTime.now().toString(),
          });
        });
      });
      _windowsController.loadingState.listen((state) {
        if (!mounted) return;
        if (state == win_web.LoadingState.navigationCompleted) {
          setState(() {
            _isLoading = false;
            if (_networkRequests.isNotEmpty) {
              _networkRequests.last['status'] = '200 OK';
            }
          });
          if (!_isDevToolsMinimized) {
            _extractDevToolsData();
          }
        }
      });
      _windowsController.webMessage.listen((msg) {
        if (!mounted) return;
        try {
          if (msg is String) {
            try {
              msg = jsonDecode(msg);
            } catch (_) {}
          }
          if (msg is Map) {
            if (msg['type'] == 'js_eval_result') {
              final id = msg['id'];
              final result = msg['result'];
              final error = msg['error'];
              if (_jsCallbacks.containsKey(id)) {
                if (error != null) {
                  _jsCallbacks[id]!.completeError(error);
                } else {
                  _jsCallbacks[id]!.complete(result);
                }
                _jsCallbacks.remove(id);
              }
              return;
            }
            if (msg['type'] == 'network') {
               setState(() {
                 _networkRequests.add({
                    'url': msg['url'] ?? '',
                    'method': msg['method'] ?? 'GET',
                    'status': msg['status'] ?? '',
                    'payload': msg['payload'] ?? '',
                    'response': msg['response'] ?? '',
                    'time': DateTime.now().toString(),
                    'contentType': msg['contentType'] ?? '',
                 });
               });
               return;
            }
            if (msg['type'] == 'console') {
               setState(() {
                  _consoleLogs.add(msg['message']);
               });
               return;
            }
          }
        } catch (e) {
          debugPrint('Error parsing Windows WebMessage: $e');
        }
      });
      
      setState(() {
        _isWindowsWebviewInitialized = true;
      });
      
      if (widget.initialUrl != null && widget.initialUrl!.isNotEmpty) {
        await _windowsController.loadUrl(widget.initialUrl!);
      } else {
        await _windowsController.loadStringContent(_getDefaultHtml());
      }
    } catch (e) {
      debugPrint('Windows Webview Initialization Error: $e');
    }
  }

  Future<void> _runJavascript(String code) async {
    if (Platform.isWindows) {
      if (!_isWindowsWebviewInitialized) return;
      await _windowsController.executeScript(code);
    } else {
      try {
        await _controller?.evaluateJavascript(source: code);
      } catch (_) {}
    }
  }

  Future<dynamic> _evaluateJavascript(String code) async {
    if (Platform.isWindows) {
      if (!_isWindowsWebviewInitialized) return null;
      final id = const Uuid().v4();
      final completer = Completer<dynamic>();
      _jsCallbacks[id] = completer;
      final wrappedCode = '''
        (function() {
          try {
            var result = eval(${jsonEncode(code)});
            window.chrome.webview.postMessage({
              type: 'js_eval_result',
              id: '$id',
              result: result,
              error: null
            });
          } catch(e) {
            window.chrome.webview.postMessage({
              type: 'js_eval_result',
              id: '$id',
              result: null,
              error: e.toString()
            });
          }
        })();
      ''';
      await _windowsController.executeScript(wrappedCode);
      return completer.future.timeout(const Duration(seconds: 2), onTimeout: () {
        _jsCallbacks.remove(id);
        return null;
      });
    } else {
      try {
        return await _controller?.evaluateJavascript(source: code);
      } catch (_) {
        return null;
      }
    }
  }

  void _goBack() {
    if (Platform.isWindows) {
      if (_isWindowsWebviewInitialized) _windowsController.goBack();
    } else {
      _controller?.goBack();
    }
  }

  void _goForward() {
    if (Platform.isWindows) {
      if (_isWindowsWebviewInitialized) _windowsController.goForward();
    } else {
      _controller?.goForward();
    }
  }

  void _reload() {
    if (Platform.isWindows) {
      if (_isWindowsWebviewInitialized) _windowsController.reload();
    } else {
      _controller?.reload();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Address Bar
        Container(
          height: 48,
          color: const Color(0xFF333333),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              IconButton(
                icon: HugeIcon(icon: HugeIcons.strokeRoundedArrowLeft01, size: 20, color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _goBack(),
              ),
              IconButton(
                icon: HugeIcon(icon: HugeIcons.strokeRoundedArrowRight01, size: 20, color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _goForward(),
              ),
              IconButton(
                icon: HugeIcon(icon: HugeIcons.strokeRoundedRefresh, size: 20, color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _reload(),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    color: const Color(0xFF252526),
                    borderRadius: BorderRadius.circular(6),
                    border: Border.all(color: const Color(0xFF434343)),
                  ),
                  child: TextField(
                    controller: _urlController,
                    style: const TextStyle(color: Colors.white, fontSize: 13),
                    decoration: const InputDecoration(
                      filled: false,
                      hintText: 'Cari atau masukkan alamat website...',
                      hintStyle: TextStyle(color: Colors.white38),
                      border: InputBorder.none,
                      focusedBorder: InputBorder.none,
                      enabledBorder: InputBorder.none,
                      contentPadding: EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 8,
                      ),
                      isDense: true,
                    ),
                    onSubmitted: _loadUrl,
                  ),
                ),
              ),
            ],
          ),
        ),
        if (_isLoading)
          LinearProgressIndicator(
            value: _progress,
            backgroundColor: Colors.transparent,
            color: const Color(0xFF2568E7),
            minHeight: 2,
          ),

        // Split View: Webview & DevTools
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Browser View
              Expanded(
                flex: 3,
                child: Platform.isWindows
                    ? (_isWindowsWebviewInitialized ? win_web.Webview(_windowsController) : const Center(child: CircularProgressIndicator()))
                    : InAppWebView(
                      initialUrlRequest: URLRequest(url: WebUri(widget.initialUrl ?? 'about:blank')),
                      initialSettings: _settings,
                      initialUserScripts: UnmodifiableListView<UserScript>([
                        UserScript(
                          source: '''
                            // Disable View Transitions API globally before any script runs
                            if (typeof document !== 'undefined') {
                              document.startViewTransition = undefined;
                            }
                            if (typeof window !== 'undefined') {
                              window.document.startViewTransition = undefined;
                            }

                            window.ConsoleChannel = {
                              postMessage: function(msg) {
                                window.flutter_inappwebview.callHandler('ConsoleChannel', msg);
                              }
                            };
                            window.NetworkChannel = {
                              postMessage: function(msg) {
                                window.flutter_inappwebview.callHandler('NetworkChannel', msg);
                              }
                            };
                          ''',
                          injectionTime: UserScriptInjectionTime.AT_DOCUMENT_START,
                          forMainFrameOnly: false,
                        )
                      ]),
                      onWebViewCreated: (controller) {
                        _controller = controller;
                        _controller?.addJavaScriptHandler(
                          handlerName: 'ConsoleChannel',
                          callback: (args) {
                            if (!mounted) return;
                            setState(() {
                              _consoleLogs.add(args[0]);
                            });
                          },
                        );
                        _controller?.addJavaScriptHandler(
                          handlerName: 'NetworkChannel',
                          callback: (args) {
                            if (!mounted) return;
                            try {
                              final data = jsonDecode(args[0]) as Map<String, dynamic>;
                              setState(() {
                                _networkRequests.add({
                                  'url': data['url'] ?? '',
                                  'method': data['method'] ?? 'GET',
                                  'status': data['status'] ?? '',
                                  'payload': data['payload'] ?? '',
                                  'response': data['response'] ?? '',
                                  'time': DateTime.now().toString(),
                                  'contentType': data['contentType'] ?? '',
                                });
                              });
                            } catch (_) {}
                          },
                        );
                      },
                      onLoadStart: (controller, url) {
                        if (!mounted) return;
                        setState(() {
                          _isLoading = true;
                          _urlController.text = url?.toString() ?? '';
                          _consoleLogs.clear();
                          _networkRequests.add({
                            'url': _urlController.text,
                            'method': 'GET',
                            'status': 'Pending',
                            'time': DateTime.now().toString(),
                          });
                        });
                      },
                      onLoadStop: (controller, url) async {
                        if (!mounted) return;
                        setState(() {
                          _isLoading = false;
                          if (_networkRequests.isNotEmpty) {
                            _networkRequests.last['status'] = '200 OK';
                          }
                        });
                        if (!_isDevToolsMinimized) {
                          _extractDevToolsData();
                        }
                      },
                      onProgressChanged: (controller, progress) {
                        if (!mounted) return;
                        setState(() {
                          _progress = progress / 100;
                        });
                      },
                      onReceivedError: (controller, request, error) {
                        if (!mounted) return;
                        setState(() {
                          _isLoading = false;
                          _consoleLogs.add('[ERROR] ${error.description}');
                        });
                      },
                    ),
              ),
              Container(height: 1, color: const Color(0xFF333333)),
              // DevTools View
              if (_isDevToolsMinimized)
                SizedBox(
                  height: 35,
                  child: DevToolsPanel(
                    isMinimized: true,
                    onToggleMinimize: () =>
                        setState(() => _isDevToolsMinimized = false),
                    pageSource: _pageSource,
                    consoleLogs: _consoleLogs,
                    networkRequests: _networkRequests,
                    cookies: _cookies,
                    localStorage: _localStorage,
                    onClearConsole: () => setState(() => _consoleLogs.clear()),
                    onClearNetwork: () =>
                        setState(() => _networkRequests.clear()),
                    onUpdateCookie: (key, value) async {
                      await _runJavascript(
                        'document.cookie = "$key=${Uri.encodeComponent(value)}; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onDeleteCookie: (key) async {
                      await _runJavascript(
                        'document.cookie = "$key=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onUpdateLocalStorage: (key, value) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      final safeValue = value.replaceAll("'", "\\'");
                      await _runJavascript(
                        "localStorage.setItem('$safeKey', '$safeValue');",
                      );
                      _extractDevToolsData();
                    },
                    onDeleteLocalStorage: (key) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      await _runJavascript(
                        "localStorage.removeItem('$safeKey');",
                      );
                      _extractDevToolsData();
                    },
                    onExecuteJS: (code) async {},
                  ),
                )
              else
                Expanded(
                  flex: 2,
                  child: DevToolsPanel(
                    isMinimized: false,
                    onToggleMinimize: () =>
                        setState(() => _isDevToolsMinimized = true),
                    pageSource: _pageSource,
                    consoleLogs: _consoleLogs,
                    networkRequests: _networkRequests,
                    cookies: _cookies,
                    localStorage: _localStorage,
                    onClearConsole: () => setState(() => _consoleLogs.clear()),
                    onClearNetwork: () =>
                        setState(() => _networkRequests.clear()),
                    onUpdateCookie: (key, value) async {
                      await _runJavascript(
                        'document.cookie = "$key=${Uri.encodeComponent(value)}; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onDeleteCookie: (key) async {
                      await _runJavascript(
                        'document.cookie = "$key=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onUpdateLocalStorage: (key, value) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      final safeValue = value.replaceAll("'", "\\'");
                      await _runJavascript(
                        "localStorage.setItem('$safeKey', '$safeValue');",
                      );
                      _extractDevToolsData();
                    },
                    onDeleteLocalStorage: (key) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      await _runJavascript(
                        "localStorage.removeItem('$safeKey');",
                      );
                      _extractDevToolsData();
                    },
                    onExecuteJS: (code) async {
                      try {
                        final result = await _evaluateJavascript(code);
                        if (mounted) {
                          setState(() {
                            _consoleLogs.add('> $code');
                            _consoleLogs.add('< $result');
                          });
                        }
                      } catch (e) {
                        if (mounted) {
                          setState(() {
                            _consoleLogs.add('> $code');
                            if (e.toString().contains(
                                  'returned a `null` value',
                                ) ||
                                e.toString().contains(
                                  'returned a \'null\' value',
                                )) {
                              _consoleLogs.add('< undefined');
                            } else {
                              _consoleLogs.add('[ERROR] $e');
                            }
                          });
                        }
                      }
                    },
                  ),
                ),
            ],
          ),
        ),
      ],
    );
  }
}
