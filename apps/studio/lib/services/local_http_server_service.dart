import 'dart:io';
import 'package:path/path.dart' as p;

class LocalHttpServerService {
  HttpServer? _server;
  Process? _phpProcess;
  String? _rootPath;
  int? _port;
  bool _isRunning = false;

  bool get isRunning => _isRunning;
  int? get port => _port;

  Future<void> start(String rootPath, {int defaultPort = 8000}) async {
    if (_isRunning) return;

    _rootPath = rootPath;
    
    try {
      _server = await HttpServer.bind(InternetAddress.loopbackIPv4, defaultPort);
    } catch (e) {
      // Jika port terpakai, sistem akan mencari port kosong otomatis (port 0)
      _server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    }

    _port = _server!.port;
    _isRunning = true;

    _server!.listen((HttpRequest request) {
      _handleRequest(request);
    });
  }

  Future<void> startPhpServer(String rootPath, String phpExecutable, {int defaultPort = 8000}) async {
    if (_isRunning) return;

    _rootPath = rootPath;

    // Otomatis deteksi WordPress dan buat mu-plugin untuk mematikan View Transitions
    // (Mencegah Android WebView crash/blank screen)
    if (File(p.join(rootPath, 'wp-config.php')).existsSync() || 
        File(p.join(rootPath, 'wp-config-sample.php')).existsSync()) {
      try {
        final muDir = Directory(p.join(rootPath, 'wp-content', 'mu-plugins'));
        if (!muDir.existsSync()) {
          muDir.createSync(recursive: true);
        }
        final fixFile = File(p.join(muDir.path, 'rpl_studio_compat.php'));
        fixFile.writeAsStringSync('''<?php
/**
 * Plugin Name: RPL Studio Compatibility Fix
 * Description: Disables CSS View Transitions and polyfills to prevent Android WebView crashes on navigation.
 * Author: RPL Studio
 */
add_action('init', function() {
    remove_action('wp_head', 'wp_view_transition_meta');
    remove_action('admin_head', 'wp_view_transition_meta');
});
add_action('admin_head', function() {
    echo "<style>@view-transition { navigation: none !important; }</style>";
    echo "<script>if (typeof document !== 'undefined') document.startViewTransition = undefined;</script>";
}, 1);
add_action('wp_head', function() {
    echo "<style>@view-transition { navigation: none !important; }</style>";
    echo "<script>if (typeof document !== 'undefined') document.startViewTransition = undefined;</script>";
}, 1);
''');
      } catch (_) {}
    }
    
    // Cari port yang kosong
    int targetPort = defaultPort;
    try {
      final s = await ServerSocket.bind(InternetAddress.loopbackIPv4, defaultPort);
      targetPort = s.port;
      await s.close();
    } catch (e) {
      final s = await ServerSocket.bind(InternetAddress.loopbackIPv4, 0);
      targetPort = s.port;
      await s.close();
    }

    _port = targetPort;
    _isRunning = true;

    try {
      _phpProcess = await Process.start(
        phpExecutable, 
        ['-d', 'opcache.enable=0', '-d', 'opcache.enable_cli=0', '-d', 'error_reporting=22527', '-S', '0.0.0.0:$_port', '-t', rootPath],
        environment: {
          'TMPDIR': Directory.systemTemp.path,
        },
      );
    } catch (e) {
      _isRunning = false;
      _port = null;
      rethrow;
    }
  }

  Future<void> stop() async {
    if (!_isRunning) return;
    
    if (_server != null) {
      await _server!.close(force: true);
      _server = null;
    }
    
    if (_phpProcess != null) {
      _phpProcess!.kill();
      _phpProcess = null;
    }
    
    _port = null;
    _isRunning = false;
  }

  void _handleRequest(HttpRequest request) async {
    final response = request.response;
    
    // Tambahkan header CORS
    response.headers.add('Access-Control-Allow-Origin', '*');
    // Matikan cache agar reload langsung terasa
    response.headers.add('Cache-Control', 'no-cache, no-store, must-revalidate');

    if (request.method != 'GET') {
      response.statusCode = HttpStatus.methodNotAllowed;
      await response.close();
      return;
    }

    var path = request.uri.path;
    if (path == '/') {
      path = '/index.html';
    }

    // Cegah path traversal
    if (path.contains('..')) {
      response.statusCode = HttpStatus.forbidden;
      await response.close();
      return;
    }

    final file = File(p.join(_rootPath!, path.substring(1)));

    if (!await file.exists()) {
      response.statusCode = HttpStatus.notFound;
      response.write('404 Not Found');
      await response.close();
      return;
    }

    try {
      final ext = p.extension(file.path).toLowerCase();
      response.headers.contentType = _getContentType(ext);

      await file.openRead().pipe(response);
    } catch (e) {
      response.statusCode = HttpStatus.internalServerError;
      response.write('500 Internal Server Error: $e');
      await response.close();
    }
  }

  ContentType _getContentType(String ext) {
    switch (ext) {
      case '.html':
        return ContentType.html;
      case '.css':
        return ContentType('text', 'css', charset: 'utf-8');
      case '.js':
        return ContentType('application', 'javascript', charset: 'utf-8');
      case '.json':
        return ContentType.json;
      case '.png':
        return ContentType('image', 'png');
      case '.jpg':
      case '.jpeg':
        return ContentType('image', 'jpeg');
      case '.gif':
        return ContentType('image', 'gif');
      case '.svg':
        return ContentType('image', 'svg+xml');
      case '.ico':
        return ContentType('image', 'x-icon');
      case '.txt':
        return ContentType.text;
      case '.mp3':
        return ContentType('audio', 'mpeg');
      case '.mp4':
        return ContentType('video', 'mp4');
      default:
        return ContentType.binary;
    }
  }
}
