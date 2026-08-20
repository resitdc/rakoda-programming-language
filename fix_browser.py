import re

with open('apps/studio/lib/features/browser/browser_workspace.dart', 'r') as f:
    content = f.read()

# Add import dart:collection
if "import 'dart:collection';" not in content:
    content = content.replace("import 'dart:io';", "import 'dart:io';\nimport 'dart:collection';")

# Fix _controller.loadHtmlString to _controller?.loadData(data: ...)
content = re.sub(
    r"_controller\.loadHtmlString\((.*?)\)",
    r"_controller?.loadData(data: \1)",
    content
)

# Fix _controller.loadRequest(Uri.parse(...)) to _controller?.loadUrl(urlRequest: URLRequest(url: WebUri(...)))
content = re.sub(
    r"_controller\.loadRequest\(Uri\.parse\((.*?)\)\)",
    r"_controller?.loadUrl(urlRequest: URLRequest(url: WebUri(\1)))",
    content
)

# Fix _controller.runJavaScript to _controller?.evaluateJavascript(source: ...)
content = re.sub(
    r"_controller\.runJavaScript\((.*?)\)",
    r"_controller?.evaluateJavascript(source: \1)",
    content
)

# Fix _controller.runJavaScriptReturningResult to _controller?.evaluateJavascript(source: ...)
content = re.sub(
    r"_controller\.runJavaScriptReturningResult\((.*?)\)",
    r"_controller?.evaluateJavascript(source: \1)",
    content
)

# Fix _controller.goBack, goForward, reload
content = re.sub(r"_controller\.goBack\(\)", r"_controller?.goBack()", content)
content = re.sub(r"_controller\.goForward\(\)", r"_controller?.goForward()", content)
content = re.sub(r"_controller\.reload\(\)", r"_controller?.reload()", content)

with open('apps/studio/lib/features/browser/browser_workspace.dart', 'w') as f:
    f.write(content)

