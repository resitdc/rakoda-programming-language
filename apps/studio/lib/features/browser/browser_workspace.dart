import 'dart:convert';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:webview_flutter/webview_flutter.dart';
import 'package:webview_flutter_wkwebview/webview_flutter_wkwebview.dart';
import 'devtools_panel.dart';

class BrowserWorkspace extends StatefulWidget {
  const BrowserWorkspace({super.key});

  @override
  State<BrowserWorkspace> createState() => _BrowserWorkspaceState();
}

class _BrowserWorkspaceState extends State<BrowserWorkspace> {
  late final WebViewController _controller;
  final TextEditingController _urlController = TextEditingController(
    text: 'https://flutter.dev',
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
  void initState() {
    super.initState();
    if (!kIsWeb && Platform.isMacOS) {
      WebViewPlatform.instance = WebKitWebViewPlatform();
    }

    _controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted);

    try {
      _controller.setBackgroundColor(const Color(0xFF1E1E1E));
    } catch (_) {
      // Ignored: webview_flutter_wkwebview throws UnimplementedError for opaque on macOS
    }

    _controller
      ..setNavigationDelegate(
        NavigationDelegate(
          onProgress: (int progress) {
            if (!mounted) return;
            setState(() {
              _progress = progress / 100;
            });
          },
          onPageStarted: (String url) {
            if (!mounted) return;
            setState(() {
              _isLoading = true;
              _urlController.text =
                  (url.startsWith('data:text/html') || url == 'about:blank')
                  ? 'rpl://browser'
                  : url;
              _consoleLogs.clear();
              _networkRequests.add({
                'url': url,
                'method': 'GET',
                'status': 'Pending',
                'time': DateTime.now().toString(),
              });
            });
          },
          onPageFinished: (String url) async {
            if (!mounted) return;
            setState(() {
              _isLoading = false;
              if (_networkRequests.isNotEmpty) {
                _networkRequests.last['status'] = '200 OK';
              }
            });
            _extractDevToolsData();
          },
          onWebResourceError: (WebResourceError error) {
            if (!mounted) return;
            setState(() {
              _isLoading = false;
              _consoleLogs.add('[ERROR] ${error.description}');
            });
          },
        ),
      )
      ..addJavaScriptChannel(
        'ConsoleChannel',
        onMessageReceived: (JavaScriptMessage message) {
          if (!mounted) return;
          setState(() {
            _consoleLogs.add(message.message);
          });
        },
      )
      ..addJavaScriptChannel(
        'NetworkChannel',
        onMessageReceived: (JavaScriptMessage message) {
          if (!mounted) return;
          try {
            final data = jsonDecode(message.message) as Map<String, dynamic>;
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
          } catch (e) {
            debugPrint('Error parsing network message: $e');
          }
        },
      )
      ..setOnJavaScriptAlertDialog((
        JavaScriptAlertDialogRequest request,
      ) async {
        if (!mounted) return;
        await showDialog(
          context: context,
          builder: (context) => AlertDialog(
            backgroundColor: const Color(0xFF252526),
            title: const Text(
              'JavaScript Alert',
              style: TextStyle(color: Colors.white),
            ),
            content: Text(
              request.message,
              style: const TextStyle(color: Colors.white70),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text(
                  'OK',
                  style: TextStyle(color: Color(0xFF007ACC)),
                ),
              ),
            ],
          ),
        );
      })
      ..setOnJavaScriptConfirmDialog((
        JavaScriptConfirmDialogRequest request,
      ) async {
        if (!mounted) return false;
        final result = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            backgroundColor: const Color(0xFF252526),
            title: const Text(
              'JavaScript Confirm',
              style: TextStyle(color: Colors.white),
            ),
            content: Text(
              request.message,
              style: const TextStyle(color: Colors.white70),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(false),
                child: const Text(
                  'Cancel',
                  style: TextStyle(color: Colors.white54),
                ),
              ),
              TextButton(
                onPressed: () => Navigator.of(context).pop(true),
                child: const Text(
                  'OK',
                  style: TextStyle(color: Color(0xFF007ACC)),
                ),
              ),
            ],
          ),
        );
        return result ?? false;
      })
      ..setOnJavaScriptTextInputDialog((
        JavaScriptTextInputDialogRequest request,
      ) async {
        if (!mounted) return '';
        final TextEditingController textController = TextEditingController(
          text: request.defaultText,
        );
        final result = await showDialog<String>(
          context: context,
          builder: (context) => AlertDialog(
            backgroundColor: const Color(0xFF252526),
            title: const Text(
              'JavaScript Prompt',
              style: TextStyle(color: Colors.white),
            ),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  request.message,
                  style: const TextStyle(color: Colors.white70),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: textController,
                  style: const TextStyle(color: Colors.white),
                  decoration: const InputDecoration(
                    enabledBorder: OutlineInputBorder(
                      borderSide: BorderSide(color: Color(0xFF333333)),
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderSide: BorderSide(color: Color(0xFF007ACC)),
                    ),
                  ),
                ),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(null),
                child: const Text(
                  'Cancel',
                  style: TextStyle(color: Colors.white54),
                ),
              ),
              TextButton(
                onPressed: () => Navigator.of(context).pop(textController.text),
                child: const Text(
                  'OK',
                  style: TextStyle(color: Color(0xFF007ACC)),
                ),
              ),
            ],
          ),
        );
        return result ?? '';
      })
      ..loadHtmlString(_getDefaultHtml());
  }

  Future<void> _extractDevToolsData() async {
    try {
      // Inject console override to capture logs
      await _controller.runJavaScript('''
        if (!window._consoleOverridden) {
          window._consoleOverridden = true;
          const oldLog = console.log;
          const oldError = console.error;
          const oldWarn = console.warn;
          console.log = function(...args) {
            ConsoleChannel.postMessage('[LOG] ' + args.join(' '));
            oldLog.apply(console, args);
          };
          console.error = function(...args) {
            ConsoleChannel.postMessage('[ERROR] ' + args.join(' '));
            oldError.apply(console, args);
          };
          console.warn = function(...args) {
            ConsoleChannel.postMessage('[WARN] ' + args.join(' '));
            oldWarn.apply(console, args);
          };
        }
      ''');

      // Inject network interceptor
      await _controller.runJavaScript('''
        if (!window._networkOverridden) {
          window._networkOverridden = true;

          // 1. Intercept Fetch
          const originalFetch = window.fetch;
          window.fetch = async function(...args) {
            const url = args[0];
            const options = args[1] || {};
            const method = options.method || 'GET';
            const payload = options.body ? options.body.toString() : '';

            NetworkChannel.postMessage(JSON.stringify({
              url: url,
              method: method,
              status: 'Pending',
              payload: payload,
              response: '',
              contentType: 'application/json'
            }));

            try {
              const response = await originalFetch(...args);
              const clone = response.clone();
              let responseText = '';
              try {
                responseText = await clone.text();
              } catch (_) {}
              
              NetworkChannel.postMessage(JSON.stringify({
                url: url,
                method: method,
                status: response.status + ' ' + response.statusText,
                payload: payload,
                response: responseText,
                contentType: response.headers.get('content-type') || 'application/json'
              }));
              return response;
            } catch (err) {
              NetworkChannel.postMessage(JSON.stringify({
                url: url,
                method: method,
                status: 'Failed',
                payload: payload,
                response: err.toString(),
                contentType: 'text/plain'
              }));
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

            NetworkChannel.postMessage(JSON.stringify({
              url: url,
              method: method,
              status: 'Pending',
              payload: payload,
              response: '',
              contentType: 'text/plain'
            }));

            xhr.addEventListener('load', function() {
              NetworkChannel.postMessage(JSON.stringify({
                url: url,
                method: method,
                status: xhr.status + ' ' + xhr.statusText,
                payload: payload,
                response: xhr.responseText,
                contentType: xhr.getResponseHeader('content-type') || 'text/plain'
              }));
            });

            xhr.addEventListener('error', function() {
              NetworkChannel.postMessage(JSON.stringify({
                url: url,
                method: method,
                status: 'Failed',
                payload: payload,
                response: 'Network Error',
                contentType: 'text/plain'
              }));
            });

            return origSend.apply(this, arguments);
          };
        }
      ''');

      // 3. Inject Resource Timing collector for CSS, JS, Images, Media
      await _controller.runJavaScript('''
        (function() {
          const resources = performance.getEntriesByType('resource');
          for (const res of resources) {
            if (res.initiatorType !== 'xmlhttprequest' && res.initiatorType !== 'fetch') {
              let cType = 'text/plain';
              if (res.initiatorType === 'css') cType = 'text/css';
              else if (res.initiatorType === 'img') cType = 'image/png';
              else if (res.initiatorType === 'script') cType = 'text/javascript';

              NetworkChannel.postMessage(JSON.stringify({
                url: res.name,
                method: 'GET',
                status: '200 OK',
                payload: '',
                response: '[Resource Loaded from Cache/Network]',
                contentType: cType
              }));
            }
          }
        })();
      ''');

      // Get page source
      final html = await _controller.runJavaScriptReturningResult(
        'document.documentElement.outerHTML',
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
      final cookiesObj = await _controller.runJavaScriptReturningResult(
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
      final lsObj = await _controller.runJavaScriptReturningResult(
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
      debugPrint('Failed to extract devtools data: $e');
    }
  }

  void _loadUrl(String url) {
    url = url.trim();
    if (url.isEmpty) return;
    if (url == 'rpl://browser') {
      _controller.loadHtmlString(_getDefaultHtml());
      FocusScope.of(context).unfocus();
      return;
    }

    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      if (url.contains('.') && !url.contains(' ')) {
        url = 'https://$url';
      } else {
        url = 'https://www.google.com/search?q=${Uri.encodeComponent(url)}';
      }
    }
    _controller.loadRequest(Uri.parse(url));
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
      color: #007acc;
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
      border-color: #007acc;
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
                icon: const Icon(
                  Icons.arrow_back,
                  size: 20,
                  color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _controller.goBack(),
              ),
              IconButton(
                icon: const Icon(
                  Icons.arrow_forward,
                  size: 20,
                  color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _controller.goForward(),
              ),
              IconButton(
                icon: const Icon(
                  Icons.refresh,
                  size: 20,
                  color: Colors.white70,
                ),
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32),
                onPressed: () => _controller.reload(),
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
            color: const Color(0xFF007ACC),
            minHeight: 2,
          ),

        // Split View: Webview & DevTools
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Browser View
              Expanded(flex: 3, child: WebViewWidget(controller: _controller)),
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
                      await _controller.runJavaScript(
                        'document.cookie = "$key=${Uri.encodeComponent(value)}; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onDeleteCookie: (key) async {
                      await _controller.runJavaScript(
                        'document.cookie = "$key=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onUpdateLocalStorage: (key, value) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      final safeValue = value.replaceAll("'", "\\'");
                      await _controller.runJavaScript(
                        "localStorage.setItem('$safeKey', '$safeValue');",
                      );
                      _extractDevToolsData();
                    },
                    onDeleteLocalStorage: (key) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      await _controller.runJavaScript(
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
                      await _controller.runJavaScript(
                        'document.cookie = "$key=${Uri.encodeComponent(value)}; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onDeleteCookie: (key) async {
                      await _controller.runJavaScript(
                        'document.cookie = "$key=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/";',
                      );
                      _extractDevToolsData();
                    },
                    onUpdateLocalStorage: (key, value) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      final safeValue = value.replaceAll("'", "\\'");
                      await _controller.runJavaScript(
                        "localStorage.setItem('$safeKey', '$safeValue');",
                      );
                      _extractDevToolsData();
                    },
                    onDeleteLocalStorage: (key) async {
                      final safeKey = key.replaceAll("'", "\\'");
                      await _controller.runJavaScript(
                        "localStorage.removeItem('$safeKey');",
                      );
                      _extractDevToolsData();
                    },
                    onExecuteJS: (code) async {
                      try {
                        final result = await _controller
                            .runJavaScriptReturningResult(code);
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
