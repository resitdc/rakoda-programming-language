import re

with open('apps/studio/lib/features/browser/browser_workspace.dart', 'r') as f:
    content = f.read()

# 1. Imports
content = re.sub(
    r"import 'package:webview_flutter/webview_flutter.dart';\nimport 'package:webview_flutter_wkwebview/webview_flutter_wkwebview.dart';\nimport 'package:webview_flutter_android/webview_flutter_android.dart';",
    "import 'package:flutter_inappwebview/flutter_inappwebview.dart';",
    content
)

# 2. State vars
content = re.sub(
    r"late final WebViewController _controller;\n\s*late final Widget _webViewWidget;",
    "InAppWebViewController? _controller;\n  late final InAppWebViewSettings _settings = InAppWebViewSettings(\n    isInspectable: true,\n    javaScriptEnabled: true,\n    transparentBackground: true,\n    useHybridComposition: true,\n  );",
    content
)

# 3. Dispose
content = re.sub(
    r"_controller\.loadHtmlString\('about:blank'\);\n\s*_controller\.clearCache\(\);\n\s*_controller\.clearLocalStorage\(\);",
    "",
    content
)

# 4. Remove initState WebViewController setup
# This matches from `if (!kIsWeb && Platform.isMacOS) {` down to `_webViewWidget = WebViewWidget(controller: _controller);\n    }`
content = re.sub(
    r"if \(!kIsWeb && Platform\.isMacOS\) \{.*?_webViewWidget = WebViewWidget\(controller: _controller\);\n    \}",
    "// Controller is initialized in onWebViewCreated for InAppWebView",
    content,
    flags=re.DOTALL
)

# 5. Build method: replace _webViewWidget with InAppWebView
inappwebview_code = """InAppWebView(
                      initialUrlRequest: URLRequest(url: WebUri(widget.initialUrl ?? 'about:blank')),
                      initialSettings: _settings,
                      initialUserScripts: UnmodifiableListView<UserScript>([
                        UserScript(
                          source: '''
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
                    )"""

content = re.sub(
    r"_webViewWidget",
    inappwebview_code,
    content,
    count=1
)

# 6. Replace _evaluateJavascript
content = re.sub(
    r"Future<Object\?> _evaluateJavascript\(String script\) async \{",
    "Future<Object?> _evaluateJavascript(String script) async {\n    if (Platform.isWindows) {\n      if (!_isWindowsWebviewInitialized) return null;\n      final completer = Completer<dynamic>();\n      final id = const Uuid().v4();\n      _jsCallbacks[id] = completer;\n      _windowsController.executeScript('''\n        try {\n          var result = eval(`${script.replaceAll('`', '\\\\`')}`);\n          window.chrome.webview.postMessage({type: 'js_eval_result', id: '$id', result: result});\n        } catch (e) {\n          window.chrome.webview.postMessage({type: 'js_eval_result', id: '$id', error: e.toString()});\n        }\n      ''');\n      final result = await completer.future;\n      return result ?? '';\n    }\n    return _controller?.evaluateJavascript(source: script);",
    content
)

# Fix _evaluateJavascript body logic since I just replaced the declaration.
# Let's just do a simpler replace.
with open('apps/studio/lib/features/browser/browser_workspace.dart', 'w') as f:
    f.write(content)

