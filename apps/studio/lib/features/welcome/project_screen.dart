import 'dart:io';
import 'package:flutter/services.dart';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import 'package:file_picker/file_picker.dart';
import '../../models/project.dart';
import '../../services/project_service.dart';
import '../editor/editor_tab.dart';
import '../editor/code_editor.dart';
import '../explorer/file_explorer.dart';
import 'welcome_screen.dart';
import '../editor/code_executor_service.dart';
import '../../src/rust/api/simple.dart';
import 'activity_bar.dart';
import 'search_panel.dart';
import '../browser/browser_workspace.dart';
import '../database/database_workspace.dart';
import '../http/http_workspace.dart';
import '../classroom/classroom_panel.dart';
import '../classroom/classroom_service.dart';
import '../identity/identity_service.dart';
import 'pdf_viewer_widget.dart';
import 'spreadsheet_viewer_widget.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../settings/settings_provider.dart';
import 'package:xterm/xterm.dart';
import 'package:flutter_pty/flutter_pty.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'dart:convert';
import 'package:flutter_highlight/flutter_highlight.dart';
import 'package:flutter_highlight/themes/vs2015.dart';
import '../editor/rpl_languages.dart';

enum WorkspaceType { editor, browser, database, http }

class ProjectScreen extends ConsumerStatefulWidget {
  final Project project;
  const ProjectScreen({super.key, required this.project});
  @override
  ConsumerState<ProjectScreen> createState() => _ProjectScreenState();
}

class _ProjectScreenState extends ConsumerState<ProjectScreen> {
  late List<EditorTab> _openTabs;
  int _activeTabIndex = 0;
  ActivityType? _activeActivity = ActivityType.explorer;
  bool _isTerminalMinimized = true;
  int _explorerVersion = 0;
  int? _targetLineNumber;

  WorkspaceType _activeWorkspace = WorkspaceType.editor;
  String? _browserInitialUrl;

  bool _showLocalSearch = false;
  String _localSearchQuery = '';
  final TextEditingController _localSearchController = TextEditingController();

  final ClassroomService _classroomService = ClassroomService();
  String _liveCodeContent = '';
  bool _isLiveCodeVisible = false;
  bool _isLiveCodeMinimized = false;
  int? _liveSelectionStart;
  int? _liveSelectionEnd;
  String? _liveHostName;

  late final Terminal _terminal;
  late final TerminalController _terminalController;
  Pty? _pty;
  final FocusNode _terminalFocusNode = FocusNode();

  // iOS Custom Terminal States
  final List<String> _iosTerminalLines = [];
  final TextEditingController _iosTerminalInputController = TextEditingController();
  final ScrollController _iosTerminalScrollController = ScrollController();
  late String _iosTerminalCwd;

  @override
  void initState() {
    super.initState();
    registerRplLanguages();
    final isLowEnd = ref.read(settingsProvider).isLowEndMode;
    _terminal = Terminal(maxLines: isLowEnd ? 70 : 1000);
    if (isLowEnd) {
      _terminal.setCursorBlinkMode(false);
    }
    _terminalController = TerminalController();

    if (!Platform.isIOS) {
      String shell =
          Platform.environment['SHELL'] ??
          (Platform.isWindows ? 'cmd.exe' : 'sh');
      try {
        _pty = Pty.start(
          shell,
          columns: 80,
          rows: 24,
          workingDirectory: widget.project.path,
          environment: Platform.environment,
        );

        _pty!.output
            .cast<List<int>>()
            .transform(const Utf8Decoder(allowMalformed: true))
            .listen((data) {
              _terminal.write(data);
              _checkForLocalhostUrl(data);
            });

        _terminal.onOutput = (data) {
          _pty?.write(const Utf8Encoder().convert(data));
        };

        _terminal.onResize = (w, h, pw, ph) {
          _pty?.resize(h, w);
        };
        
        _injectPathsToPty();
      } catch (e) {
        _terminal.write('\x1b[31mTerminal Error: $e\x1b[0m\r\n');
      }
    }

    if (Platform.isIOS) {
      _iosTerminalCwd = widget.project.path;
      _iosTerminalLines.add('Selamat datang di RPL Studio Terminal!');
      _iosTerminalLines.add('Ketik "help" untuk melihat daftar perintah.');
      _iosTerminalLines.add('');
    }

    final mainFile = _findMainFile();
    _openTabs = [
      EditorTab(
        filePath: mainFile.path,
        fileName: mainFile.path.split(Platform.pathSeparator).last,
        content: _readFile(mainFile.path),
      ),
    ];
    ProjectService.touchProject(widget.project);
    
    _classroomService.onMessage.listen((msg) {
      if (msg.eventType == 'live_code' && mounted) {
        setState(() {
          _liveCodeContent = msg.text;
          _liveSelectionStart = msg.selectionStart;
          _liveSelectionEnd = msg.selectionEnd;
          _liveHostName = msg.name;
          _isLiveCodeVisible = true;
        });
      } else if (msg.eventType == 'hide_live_code' && mounted) {
        setState(() {
          _isLiveCodeVisible = false;
          _liveCodeContent = '';
          _liveSelectionStart = null;
          _liveSelectionEnd = null;
          _liveHostName = null;
        });
      } else if (msg.eventType == 'terminal_output' && mounted && !_classroomService.isHost) {
        setState(() {
          _isTerminalMinimized = false; // Buka terminal jika tertutup
        });
        // Ubah warna text menjadi Cyan (\x1b[36m) agar berbeda dengan output siswa sendiri
        final coloredText = '\x1b[36m' + 
            msg.text
               .replaceAll('\x1b[0m', '\x1b[0m\x1b[36m')
               .replaceAll('\x1B[0m', '\x1B[0m\x1b[36m') + 
            '\x1b[0m';
        _terminal.write(coloredText);
      } else if (msg.eventType == 'request_code_stream' && mounted && !_classroomService.isHost) {
        if (msg.uuid == IdentityService.uuid) {
          _classroomService.isBroadcastingToHost = msg.text == 'true';
          if (_classroomService.isBroadcastingToHost && _openTabs.isNotEmpty) {
            _classroomService.sendStudentLiveCode(
              _openTabs[_activeTabIndex].content,
            );
          }
        }
      } else if (msg.eventType == 'student_live_code' && mounted && _classroomService.isHost) {
        setState(() {
          _liveCodeContent = msg.text;
          _liveSelectionStart = msg.selectionStart;
          _liveSelectionEnd = msg.selectionEnd;
          _liveHostName = "Siswa: ${msg.name}";
          _isLiveCodeVisible = true;
        });
      }
    });
    _classroomService.onStatusChanged.listen((status) {
      if (status == ClassroomStatus.disconnected && mounted) {
        setState(() {
          _isLiveCodeVisible = false;
          _liveCodeContent = '';
        });
      }
    });
  }

  void _injectPathsToPty() async {
    if (_pty == null) return;
    try {
      final nodePaths = await CodeExecutorService.getInstalledRuntimePaths('node');
      final phpPaths = await CodeExecutorService.getInstalledRuntimePaths('php');
      
      String newPath = '';
      String nodeDir = '';
      String phpDir = '';
      
      if (nodePaths.isNotEmpty) {
        nodeDir = File(nodePaths.first).parent.path;
        newPath += '$nodeDir:';
      }
      if (phpPaths.isNotEmpty) {
        phpDir = File(phpPaths.first).parent.path;
        newPath += '$phpDir:';
      }
      
      if (newPath.isNotEmpty) {
        if (Platform.isWindows) {
          final shellCmd = 'set PATH=$newPath%PATH%\r\ncls\r\n';
          _pty?.write(const Utf8Encoder().convert(shellCmd));
        } else {
          String shellCmd = 'export PATH="$newPath\$PATH"\r\n';
          if (Platform.isAndroid) {
            // Android 10+ W^X memblokir eksekusi script/binary di /data/user/0/...
            // Jadi kita harus mendefinisikan shell function agar memanggil 'sh' secara eksplisit.
            if (nodeDir.isNotEmpty) {
              shellCmd += 'node() { sh "$nodeDir/node" "\$@"; }\r\n';
              shellCmd += 'npm() { sh "$nodeDir/npm" "\$@"; }\r\n';
              shellCmd += 'npx() { sh "$nodeDir/npx" "\$@"; }\r\n';
              shellCmd += 'pnpm() { sh "$nodeDir/pnpm" "\$@"; }\r\n';
              shellCmd += 'pnpx() { sh "$nodeDir/pnpx" "\$@"; }\r\n';
            }
            if (phpDir.isNotEmpty) {
              shellCmd += 'php() { sh "$phpDir/php" "\$@"; }\r\n';
              shellCmd += 'composer() { sh "$phpDir/composer" "\$@"; }\r\n';
            }
          }
          shellCmd += 'clear\r\n';
          _pty?.write(const Utf8Encoder().convert(shellCmd));
        }
      }
    } catch (_) {}
  }

  void _checkForLocalhostUrl(String data) {
    // Regex untuk mendeteksi URL localhost/127.0.0.1 dengan port
    final regExp = RegExp(r'http:\/\/(?:localhost|127\.0\.0\.1):\d+');
    final match = regExp.firstMatch(data);
    if (match != null) {
      final url = match.group(0)!;
      
      if (_activeWorkspace != WorkspaceType.browser) {
        // Tampilkan notifikasi agar siswa tahu Web Server sudah menyala
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('🌐 Web Server terdeteksi berjalan di $url\r\nBuka tab RPL Browser untuk melihat hasilnya!'),
            backgroundColor: const Color(0xFF2568E7),
            duration: const Duration(seconds: 5),
            action: SnackBarAction(
              label: 'BUKA BROWSER',
              textColor: Colors.white,
              onPressed: () {
                setState(() {
                  _browserInitialUrl = url;
                  _activeWorkspace = WorkspaceType.browser;
                });
              },
            ),
          ),
        );
      }
    }
  }

  String _readFile(String path) {
    try {
      return File(path).readAsStringSync();
    } catch (e) {
      return '';
    }
  }

  @override
  void dispose() {
    _pty?.kill();
    _iosTerminalInputController.dispose();
    _iosTerminalScrollController.dispose();
    _localSearchController.dispose();
    _terminalFocusNode.dispose();
    super.dispose();
  }

  File _findMainFile() {
    final mainPath = '${widget.project.path}${Platform.pathSeparator}main.rpl';
    if (File(mainPath).existsSync()) return File(mainPath);
    final dir = Directory(widget.project.path);
    if (dir.existsSync()) {
      for (final entry in dir.listSync()) {
        if (entry is File && entry.path.endsWith('.rpl')) return entry;
      }
    }
    final newFile = File(mainPath);
    newFile.createSync(recursive: true);
    return newFile;
  }

  void _openFile(String path, {int? lineNumber}) {
    final existingIndex = _openTabs.indexWhere((t) => t.filePath == path);
    if (existingIndex >= 0) {
      setState(() {
        _activeTabIndex = existingIndex;
        _targetLineNumber = lineNumber;
        _showLocalSearch = false;
        _localSearchQuery = '';
        _localSearchController.clear();
      });
      return;
    }

    final isLowEndMode = ref.read(settingsProvider).isLowEndMode;
    if (isLowEndMode && _openTabs.length >= 2) {
      // Find oldest non-modified tab to close
      final tabToClose = _openTabs.indexWhere((t) => !t.isModified);
      if (tabToClose != -1) {
        _openTabs.removeAt(tabToClose);
      }
    }

    setState(() {
      _openTabs.add(
        EditorTab(
          filePath: path,
          fileName: path.split(Platform.pathSeparator).last,
          content: _readFile(path),
        ),
      );
      _activeTabIndex = _openTabs.length - 1;
      _targetLineNumber = lineNumber;
      _showLocalSearch = false;
      _localSearchQuery = '';
      _localSearchController.clear();
    });
  }

  void _closeActiveTab() {
    if (_activeTabIndex >= 0 && _activeTabIndex < _openTabs.length) {
      _closeTab(_activeTabIndex);
    }
  }

  void _closeTab(int index) async {
    final tab = _openTabs[index];
    if (tab.isModified) {
      final result = await showDialog<String>(
        context: context,
        builder: (context) => AlertDialog(
          backgroundColor: const Color(0xFF252526),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          title: const Text(
            'Simpan Perubahan?',
            style: TextStyle(color: Colors.white, fontSize: 14),
          ),
          content: Text(
            'Apakah Anda ingin menyimpan perubahan pada "${tab.fileName}"?',
            style: const TextStyle(color: Colors.white60, fontSize: 13),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, 'cancel'),
              style: TextButton.styleFrom(foregroundColor: Colors.white54),
              child: const Text('Batal'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, 'discard'),
              style: TextButton.styleFrom(foregroundColor: Colors.white54),
              child: const Text('Jangan Simpan'),
            ),
            ElevatedButton(
              onPressed: () => Navigator.pop(context, 'save'),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF2568E7),
                foregroundColor: Colors.white,
              ),
              child: const Text('Simpan'),
            ),
          ],
        ),
      );

      if (result == null || result == 'cancel') return;

      if (result == 'save') {
        try {
          final file = File(tab.filePath);
          file.writeAsStringSync(tab.content);
          tab.isModified = false;
        } catch (e) {
          if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(
                content: Text('Gagal menyimpan: $e'),
                backgroundColor: const Color(0xFF5A1D1D),
              ),
            );
          }
          return;
        }
      }
    }

    if (!mounted) return;

    if (_openTabs.length == 1) {
      final confirm = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          backgroundColor: const Color(0xFF252526),
          shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          title: const Text(
            'Konfirmasi Keluar',
            style: TextStyle(color: Colors.white, fontSize: 14),
          ),
          content: const Text(
            'Apakah kamu yakin akan keluar?\n\nJika kamu terhubung di Room, kamu akan otomatis terputus.',
            style: TextStyle(color: Colors.white60, fontSize: 13),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              style: TextButton.styleFrom(foregroundColor: Colors.white54),
              child: const Text('Batal'),
            ),
            ElevatedButton(
              onPressed: () => Navigator.pop(context, true),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFFE53935), // Warna merah untuk aksi destruktif
                foregroundColor: Colors.white,
              ),
              child: const Text('Keluar'),
            ),
          ],
        ),
      );

      if (confirm != true) {
        return;
      }
      
      // Keluar dari sesi room juga jika ada
      _classroomService.disconnect();
    }

    setState(() {
      final currentTab =
          _openTabs.isNotEmpty && _activeTabIndex < _openTabs.length
          ? _openTabs[_activeTabIndex]
          : null;
      _openTabs.remove(tab);

      if (_openTabs.isEmpty) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (_) => const WelcomeScreen()),
        );
        return;
      }

      if (currentTab == tab) {
        if (_activeTabIndex >= _openTabs.length) {
          _activeTabIndex = _openTabs.length - 1;
        }
      } else if (currentTab != null) {
        _activeTabIndex = _openTabs.indexOf(currentTab);
      }
    });
  }

  void _createFile(String parentPath) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        title: const Text(
          'File Baru',
          style: TextStyle(color: Colors.white, fontSize: 14),
        ),
        content: Theme(
          data: Theme.of(context).copyWith(
            inputDecorationTheme: const InputDecorationTheme(
              filled: true,
              fillColor: Color(0xFF1E1E1E),
              border: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF3C3C3C)),
              ),
              enabledBorder: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF3C3C3C)),
              ),
              focusedBorder: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF2568E7)),
              ),
            ),
          ),
          child: TextField(
            controller: controller,
            autofocus: true,
            style: const TextStyle(color: Colors.white, fontSize: 13),
            decoration: InputDecoration(
              hintText: 'contoh: test.rpl',
              hintStyle: TextStyle(color: Colors.white.withOpacity(0.3)),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            style: TextButton.styleFrom(foregroundColor: Colors.white54),
            child: const Text('Batal'),
          ),
          ElevatedButton(
            onPressed: () {
              final name = controller.text.trim();
              if (name.isNotEmpty) {
                try {
                  final newFile = File(
                    '$parentPath${Platform.pathSeparator}$name',
                  );
                  if (!newFile.existsSync()) {
                    newFile.createSync(recursive: true);
                    _openFile(newFile.path);
                  }
                  setState(() => _explorerVersion++);
                } catch (e) {
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(SnackBar(content: Text('Gagal: $e')));
                }
              }
              Navigator.pop(context);
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF2568E7),
              foregroundColor: Colors.white,
            ),
            child: const Text('Buat'),
          ),
        ],
      ),
    );
  }

  void _createFolder(String parentPath) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        title: const Text(
          'Folder Baru',
          style: TextStyle(color: Colors.white, fontSize: 14),
        ),
        content: Theme(
          data: Theme.of(context).copyWith(
            inputDecorationTheme: const InputDecorationTheme(
              filled: true,
              fillColor: Color(0xFF1E1E1E),
              border: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF3C3C3C)),
              ),
              enabledBorder: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF3C3C3C)),
              ),
              focusedBorder: OutlineInputBorder(
                borderSide: BorderSide(color: Color(0xFF2568E7)),
              ),
            ),
          ),
          child: TextField(
            controller: controller,
            autofocus: true,
            style: const TextStyle(color: Colors.white, fontSize: 13),
            decoration: InputDecoration(
              hintText: 'Nama folder',
              hintStyle: TextStyle(color: Colors.white.withOpacity(0.3)),
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            style: TextButton.styleFrom(foregroundColor: Colors.white54),
            child: const Text('Batal'),
          ),
          ElevatedButton(
            onPressed: () {
              final name = controller.text.trim();
              if (name.isNotEmpty) {
                try {
                  final newDir = Directory(
                    '$parentPath${Platform.pathSeparator}$name',
                  );
                  if (!newDir.existsSync()) {
                    newDir.createSync(recursive: true);
                  }
                  setState(() => _explorerVersion++);
                } catch (e) {
                  ScaffoldMessenger.of(
                    context,
                  ).showSnackBar(SnackBar(content: Text('Gagal: $e')));
                }
              }
              Navigator.pop(context);
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF2568E7),
              foregroundColor: Colors.white,
            ),
            child: const Text('Buat'),
          ),
        ],
      ),
    );
  }

  void _saveActiveTab() {
    if (_openTabs.isEmpty) return;
    final activeTab = _openTabs[_activeTabIndex];
    try {
      final file = File(activeTab.filePath);
      file.writeAsStringSync(activeTab.content);
      setState(() => activeTab.isModified = false);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text(
            'File berhasil disimpan',
            style: TextStyle(fontSize: 12),
          ),
          backgroundColor: Color(0xFF333333),
          duration: Duration(seconds: 1),
        ),
      );
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Gagal: $e'),
          backgroundColor: const Color(0xFF5A1D1D),
        ),
      );
    }
  }

  void _renameFileOrFolder(String oldPath, String oldName, String newName) {
    try {
      final parentDir = File(oldPath).parent.path;
      final newPath = '$parentDir${Platform.pathSeparator}$newName';

      if (Directory(oldPath).existsSync()) {
        Directory(oldPath).renameSync(newPath);
      } else if (File(oldPath).existsSync()) {
        File(oldPath).renameSync(newPath);
      }

      setState(() {
        for (int i = 0; i < _openTabs.length; i++) {
          if (_openTabs[i].filePath == oldPath) {
            _openTabs[i] = EditorTab(
              filePath: newPath,
              fileName: newName,
              content: _openTabs[i].content,
              isModified: _openTabs[i].isModified,
            );
          }
        }
        _explorerVersion++;
      });
    } catch (e) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Gagal: $e')));
    }
  }

  void _importFile(String targetDirectoryPath) async {
    try {
      final result = await FilePicker.pickFiles(allowMultiple: true);
      if (result != null && result.files.isNotEmpty) {
        int count = 0;
        for (final file in result.files) {
          final fileName = file.name;
          final destinationPath =
              '$targetDirectoryPath${Platform.pathSeparator}$fileName';

          if (file.path != null) {
            final sourceFile = File(file.path!);
            await sourceFile.copy(destinationPath);
            count++;
          } else if (file.bytes != null) {
            await File(destinationPath).writeAsBytes(file.bytes!);
            count++;
          }
        }

        setState(() => _explorerVersion++);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Berhasil mengimpor $count file'),
              backgroundColor: const Color(0xFF333333),
            ),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Gagal: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(settingsProvider);
    final mediaQuery = MediaQuery.of(context);
    final isMobile = mediaQuery.size.width < 600;
    final isBrowser = _activeWorkspace == WorkspaceType.browser;
    final isDatabase = _activeWorkspace == WorkspaceType.database;
    final isHttp = _activeWorkspace == WorkspaceType.http;
    final isKeyboardOpen = mediaQuery.viewInsets.bottom > 0;

    // Determine if any input is focused so we can show the toolbar
    final showToolbar = isMobile && isKeyboardOpen;

    void _toggleTerminal() {
      setState(() {
        _isTerminalMinimized = !_isTerminalMinimized;
      });
      if (!_isTerminalMinimized) {
        if (!isMobile) {
          _terminalFocusNode.requestFocus();
        }
      }
    }

    void _handleTerminalZoomIn() {
      final fontSize = settings.terminalFontSize;
      ref.read(settingsProvider.notifier).setTerminalFontSize((fontSize + 1).clamp(8.0, 48.0));
    }

    void _handleTerminalZoomOut() {
      final fontSize = settings.terminalFontSize;
      ref.read(settingsProvider.notifier).setTerminalFontSize((fontSize - 1).clamp(8.0, 48.0));
    }

    return CallbackShortcuts(
      bindings: <ShortcutActivator, VoidCallback>{
        const SingleActivator(LogicalKeyboardKey.keyJ, control: true): _toggleTerminal,
        const SingleActivator(LogicalKeyboardKey.keyJ, meta: true): _toggleTerminal,
        const SingleActivator(LogicalKeyboardKey.equal, control: true): _handleTerminalZoomIn,
        const SingleActivator(LogicalKeyboardKey.equal, meta: true): _handleTerminalZoomIn,
        const SingleActivator(LogicalKeyboardKey.numpadAdd, control: true): _handleTerminalZoomIn,
        const SingleActivator(LogicalKeyboardKey.numpadAdd, meta: true): _handleTerminalZoomIn,
        const SingleActivator(LogicalKeyboardKey.minus, control: true): _handleTerminalZoomOut,
        const SingleActivator(LogicalKeyboardKey.minus, meta: true): _handleTerminalZoomOut,
        const SingleActivator(LogicalKeyboardKey.numpadSubtract, control: true): _handleTerminalZoomOut,
        const SingleActivator(LogicalKeyboardKey.numpadSubtract, meta: true): _handleTerminalZoomOut,
        const SingleActivator(LogicalKeyboardKey.keyW, control: true): _closeActiveTab,
        const SingleActivator(LogicalKeyboardKey.keyW, meta: true): _closeActiveTab,
      },
      child: Scaffold(
        backgroundColor: const Color(0xFF1E1E1E),
        bottomNavigationBar: showToolbar ? _buildKeyboardToolbar() : null,
        body: SafeArea(
          child: Column(
            children: [
              // ═══ Title Bar / Navbar ═══
              _buildTitleBar(),
              // ═══ Main Content ═══
              Expanded(
                child: Row(
                  children: [
                    // Activity Bar
                    ActivityBar(
                      activeActivity: _activeActivity,
                      onActivitySelected: (type) {
                        setState(() {
                          if (_activeActivity == type) {
                            _activeActivity = null;
                          } else {
                            _activeActivity = type;
                            if (type == ActivityType.browser) {
                              _activeWorkspace = WorkspaceType.browser;
                            } else if (type == ActivityType.database) {
                              _activeWorkspace = WorkspaceType.database;
                            } else if (type == ActivityType.http) {
                              _activeWorkspace = WorkspaceType.http;
                            } else {
                              _activeWorkspace = WorkspaceType.editor;
                            }
                          }
                        });
                      },
                    ),
                    Expanded(
                      child: IndexedStack(
                        index: isBrowser
                            ? 1
                            : (isDatabase ? 2 : (isHttp ? 3 : 0)),
                        children: [
                          // Index 0: Editor & Terminal
                          Row(
                            children: [
                              // Side Panel (only in layout hierarchy on Desktop)
                              if (!isMobile) _buildSidePanel(),
                              // Editor + Terminal
                              Expanded(
                                child: Flex(
                                  direction: isMobile ? Axis.vertical : Axis.horizontal,
                                  children: [
                                    Expanded(
                                      flex: 1,
                                      child: Stack(
                                        children: [
                                          Column(
                                            children: [
                                        // Tab Bar
                                        EditorTabBar(
                                          tabs: _openTabs,
                                          activeIndex: _activeTabIndex,
                                          onTap: (i) {
                                            setState(() => _activeTabIndex = i);
                                            if (_classroomService.isHost && _openTabs.isNotEmpty) {
                                              _classroomService.broadcastLiveCode(_openTabs[i].content ?? '');
                                            }
                                          },
                                          onClose: _closeTab,
                                        ),
                                        // Code Editor
                                        Expanded(
                                          child: _openTabs.isNotEmpty
                                              ? _buildEditorOrViewer(
                                                  _openTabs[_activeTabIndex],
                                                )
                                              : _buildEmptyEditor(),
                                        ),
                                        // Terminal
                                        _buildTerminal(settings),
                                        // Status Bar
                                        EditorStatusBar(
                                          tab: _openTabs.isNotEmpty
                                              ? _openTabs[_activeTabIndex]
                                              : null,
                                        ),
                                      ],
                                    ),
                                    // Overlay Backdrop Scrim (on Mobile)
                                    if (isMobile && _activeActivity != null)
                                      Positioned.fill(
                                        child: GestureDetector(
                                          onTap: () => setState(
                                            () => _activeActivity = null,
                                          ),
                                          behavior: HitTestBehavior.opaque,
                                          child: Container(color: Colors.black45),
                                        ),
                                      ),
                                    // Overlay Side Panel (on Mobile)
                                    if (isMobile && _activeActivity != null)
                                      Positioned(
                                        left: 0,
                                        top: 0,
                                        bottom: 0,
                                        width: 241, // 240 panel + 1 divider
                                        child: Material(
                                          elevation: 16,
                                          color: Colors.transparent,
                                          child: _buildSidePanel(),
                                        ),
                                      ),
                                  ],
                                ),
                              ),
                              // Live Code Viewer Split
                              if (_isLiveCodeVisible && _classroomService.status == ClassroomStatus.connected)
                                _isLiveCodeMinimized
                                  ? _buildMinimizedLiveCode(isMobile)
                                  : Expanded(
                                      flex: 1,
                                      child: Container(
                                        decoration: BoxDecoration(
                                          border: Border(
                                            left: isMobile ? BorderSide.none : const BorderSide(color: Color(0xFF3C3C3C)),
                                            top: isMobile ? const BorderSide(color: Color(0xFF3C3C3C)) : BorderSide.none,
                                          ),
                                        ),
                                        child: Column(
                                        children: [
                                          Container(
                                            height: 35,
                                            color: const Color(0xFF2D2D2D),
                                            padding: const EdgeInsets.symmetric(horizontal: 12),
                                            child: Row(
                                              children: [
                                                HugeIcon(icon: HugeIcons.strokeRoundedLaptopProgramming, size: 16, color: Colors.blueAccent),
                                                const SizedBox(width: 8),
                                                const Text(
                                                  "Live Code Guru",
                                                  style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold),
                                                ),
                                                const Spacer(),
                                                IconButton(
                                                  icon: Icon(isMobile ? Icons.expand_more : Icons.chevron_right, size: 16, color: Colors.white54),
                                                  onPressed: () => setState(() => _isLiveCodeMinimized = true),
                                                )
                                              ],
                                            ),
                                          ),
                                        Expanded(
                                          child: Container(
                                            color: const Color(0xFF1E1E1E),
                                            width: double.infinity,
                                            padding: const EdgeInsets.all(8),
                                            child: SingleChildScrollView(
                                              child: Row(
                                                crossAxisAlignment: CrossAxisAlignment.start,
                                                children: [
                                                  Container(
                                                    padding: const EdgeInsets.only(top: 4, right: 8),
                                                    width: 40,
                                                    child: Text(
                                                      List.generate(
                                                        _liveCodeContent.split('\n').length,
                                                        (i) => '${i + 1}'
                                                      ).join('\n'),
                                                      textAlign: TextAlign.right,
                                                      style: const TextStyle(
                                                        color: Color(0xFF858585),
                                                        fontFamily: 'monospace',
                                                        fontSize: 13,
                                                        height: 1.5,
                                                      ),
                                                    ),
                                                  ),
                                                  Expanded(
                                                    child: Stack(
                                                      children: [
                                                        HighlightView(
                                                          _liveCodeContent,
                                                          language: 'rpl',
                                                          theme: vs2015Theme,
                                                          padding: const EdgeInsets.all(4),
                                                          textStyle: const TextStyle(
                                                            fontFamily: 'monospace',
                                                            fontSize: 13,
                                                            height: 1.5,
                                                          ),
                                                        ),
                                                        if (_liveSelectionStart != null)
                                                          Positioned.fill(
                                                            child: CustomPaint(
                                                              painter: _LiveCursorPainter(
                                                                text: _liveCodeContent,
                                                                selectionStart: _liveSelectionStart!,
                                                                selectionEnd: _liveSelectionEnd ?? _liveSelectionStart!,
                                                                hostName: _liveHostName ?? 'Guru',
                                                                textStyle: const TextStyle(
                                                                  fontFamily: 'monospace',
                                                                  fontSize: 13,
                                                                  height: 1.5,
                                                                  color: Colors.transparent,
                                                                ),
                                                              ),
                                                            ),
                                                          ),
                                                      ],
                                                    ),
                                                  ),
                                                ],
                                              ),
                                            ),
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ],
                    ),
                          // Index 1: Browser Workspace
                          ref.watch(settingsProvider).isLowEndMode &&
                                  _activeActivity != ActivityType.browser
                              ? const SizedBox() // Bebaskan memori webview saat tidak aktif
                              : const BrowserWorkspace(),
                          // Index 2: Database Workspace
                          DatabaseWorkspace(projectPath: widget.project.path),
                          // Index 3: HTTP Workspace
                          const HttpWorkspace(),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      )
    );
  }

  /// Dark title bar with project name and action buttons.
  Widget _buildTitleBar() {
    return Container(
      height: 34,
      color: const Color(0xFF323233),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: Row(
        children: [
          IconButton(
            icon: HugeIcon(
              icon: HugeIcons.strokeRoundedArrowLeft01,
              size: 13,
              color: Colors.white54,
            ),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
            tooltip: 'Kembali ke Welcome',
            onPressed: () async {
              final confirm = await showDialog<bool>(
                context: context,
                builder: (context) => AlertDialog(
                  backgroundColor: const Color(0xFF252526),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                  title: const Text(
                    'Konfirmasi Keluar',
                    style: TextStyle(color: Colors.white, fontSize: 14),
                  ),
                  content: const Text(
                    'Apakah kamu yakin akan keluar?\n\nJika kamu terhubung di Room, kamu akan otomatis terputus.',
                    style: TextStyle(color: Colors.white60, fontSize: 13),
                  ),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.pop(context, false),
                      style: TextButton.styleFrom(foregroundColor: Colors.white54),
                      child: const Text('Batal'),
                    ),
                    ElevatedButton(
                      onPressed: () => Navigator.pop(context, true),
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFFE53935), // Warna merah untuk aksi destruktif
                        foregroundColor: Colors.white,
                      ),
                      child: const Text('Keluar'),
                    ),
                  ],
                ),
              );

              if (confirm == true) {
                // Keluar dari sesi room juga jika ada
                _classroomService.disconnect();
                
                if (context.mounted) {
                  Navigator.pushReplacement(
                    context,
                    MaterialPageRoute(builder: (_) => const WelcomeScreen()),
                  );
                }
              }
            },
          ),
          const SizedBox(width: 4),
          // Project name
          Expanded(
            child: Center(
              child: _showLocalSearch
                  ? _buildLocalSearchInput()
                  : Text(
                      widget.project.name,
                      style: const TextStyle(
                        fontSize: 12,
                        color: Colors.white60,
                        fontWeight: FontWeight.w400,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
            ),
          ),
          // Action buttons
          if (_activeWorkspace == WorkspaceType.editor) ...[
            if (_openTabs.isNotEmpty) ...[
              ValueListenableBuilder<UndoHistoryValue>(
                valueListenable: _openTabs[_activeTabIndex].undoController,
                builder: (context, value, child) {
                  return Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      _TitleBarButton(
                        icon: HugeIcons.strokeRoundedUndo,
                        tooltip: 'Undo',
                        color: value.canUndo ? Colors.white70 : Colors.white24,
                        onPressed: value.canUndo
                            ? () =>
                                  _openTabs[_activeTabIndex].undoController.undo()
                            : null,
                      ),
                      _TitleBarButton(
                        icon: HugeIcons.strokeRoundedRedo,
                        tooltip: 'Redo',
                        color: value.canRedo ? Colors.white70 : Colors.white24,
                        onPressed: value.canRedo
                            ? () =>
                                  _openTabs[_activeTabIndex].undoController.redo()
                            : null,
                      ),
                    ],
                  );
                },
              ),
              _TitleBarButton(
                icon: Icons.save_outlined,
                tooltip: 'Simpan',
                isActive: _openTabs[_activeTabIndex].isModified,
                onPressed: _openTabs[_activeTabIndex].isModified
                    ? _saveActiveTab
                    : null,
              ),
            ],
            _TitleBarButton(
              icon: Icons.play_arrow,
              tooltip: 'Run',
              color: const Color(0xFF4EC9B0),
              onPressed: () async {
                if (_openTabs.isEmpty) return;
                setState(() {
                  _isTerminalMinimized = false;
                });
                final tab = _openTabs[_activeTabIndex];
                final fileName = tab.fileName;
                final ext = fileName.contains('.') ? fileName.split('.').last.toLowerCase() : '';
                final content = tab.content;

                final cmdString = '\r\n>_ run $fileName\r\n';
                _terminal.write(cmdString);
                
                String resultString = '';
                
                if (ext == 'php' || ext == 'js') {
                  final language = ext == 'js' ? 'node' : 'php';
                  final availableRuntimes = await CodeExecutorService.getInstalledRuntimePaths(language);
                  
                  if (availableRuntimes.isEmpty) {
                    resultString = 'Error: Runtime ${language.toUpperCase()} belum terpasang. Silakan unduh melalui Pengelola Runtime.\r\n';
                  } else {
                    String selectedExe = availableRuntimes.first;
                    
                    if (availableRuntimes.length > 1 && mounted) {
                      final selected = await showDialog<String>(
                        context: context,
                        builder: (ctx) {
                          return AlertDialog(
                            backgroundColor: const Color(0xFF252526),
                            title: Text('Pilih Versi ${language.toUpperCase()}', style: const TextStyle(color: Colors.white, fontSize: 16)),
                            content: Column(
                              mainAxisSize: MainAxisSize.min,
                              children: availableRuntimes.map((path) {
                                final parts = path.split('/');
                                final folderName = parts[parts.length - 2];
                                return ListTile(
                                  title: Text(folderName, style: const TextStyle(color: Colors.white)),
                                  trailing: const Icon(Icons.play_arrow, color: Color(0xFF4EC9B0), size: 16),
                                  onTap: () => Navigator.pop(ctx, path),
                                );
                              }).toList(),
                            ),
                            actions: [
                              TextButton(
                                onPressed: () => Navigator.pop(ctx),
                                child: const Text('Batal', style: TextStyle(color: Colors.white54)),
                              )
                            ],
                          );
                        }
                      );
                      if (selected != null) {
                        selectedExe = selected;
                      } else {
                        resultString = 'Eksekusi dibatalkan.\r\n';
                      }
                    }
                    
                    if (resultString.isEmpty) {
                      final result = await CodeExecutorService.executeWithRuntime(selectedExe, content, language);
                      resultString = result.replaceAll('\n', '\r\n') + '\r\n';
                    }
                  }
                } else {
                  // Fallback to internal Rust VM for RPL or others
                  final result = await runCode(code: content);
                  resultString = result.replaceAll('\n', '\r\n') + '\r\n';
                }
                
                if (!mounted) return;
                
                _terminal.write(resultString);
                
                if (_classroomService.isHost && _classroomService.isLiveCodeSharingEnabled) {
                  _classroomService.broadcastTerminalOutput(cmdString + resultString);
                }
              },
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildLocalSearchInput() {
    return SizedBox(
      height: 24,
      width: 260,
      child: Theme(
        data: Theme.of(context).copyWith(
          inputDecorationTheme: const InputDecorationTheme(
            border: InputBorder.none,
            enabledBorder: InputBorder.none,
            focusedBorder: InputBorder.none,
            filled: true,
            fillColor: Color(0xFF3C3C3C),
          ),
        ),
        child: TextField(
          controller: _localSearchController,
          autofocus: true,
          style: const TextStyle(fontSize: 12, color: Colors.white),
          decoration: InputDecoration(
            hintText: 'Cari di file ini...',
            hintStyle: TextStyle(color: Colors.white.withOpacity(0.35)),
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 8,
              vertical: 0,
            ),
            prefixIcon: Center(
              widthFactor: 1,
              heightFactor: 1,
              child: HugeIcon(
                icon: HugeIcons.strokeRoundedSearch01,
                size: 14,
                color: Colors.white38,
              ),
            ),
            prefixIconConstraints: const BoxConstraints(minWidth: 28),
            suffixIcon: IconButton(
              icon: HugeIcon(
                icon: HugeIcons.strokeRoundedCancel01,
                size: 12,
                color: Colors.white54,
              ),
              onPressed: () {
                setState(() {
                  _showLocalSearch = false;
                  _localSearchQuery = '';
                  _localSearchController.clear();
                });
              },
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
            ),
          ),
          onChanged: (val) => setState(() => _localSearchQuery = val),
        ),
      ),
    );
  }

  /// Side panel (explorer / search / coming soon).
  Widget _buildSidePanel() {
    if (_activeActivity == null) return const SizedBox.shrink();

    Widget panel;
    switch (_activeActivity!) {
      case ActivityType.database:
        // Database has its own built-in sidebar, this shouldn't be reached
        return const SizedBox.shrink();
      case ActivityType.explorer:
        panel = DropTarget(
          onDragDone: (details) async {
            try {
              for (final file in details.files) {
                final sourceFile = File(file.path);
                final fileName = sourceFile.uri.pathSegments.last;
                final destPath =
                    '${widget.project.path}${Platform.pathSeparator}$fileName';
                await sourceFile.copy(destPath);
              }
              setState(() => _explorerVersion++);
            } catch (e) {
              if (mounted) {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(content: Text('Gagal menyalin file: $e')),
                );
              }
            }
          },
          child: FileExplorer(
            refreshTrigger: _explorerVersion,
            rootPath: widget.project.path,
            onCreateFile: _createFile,
            onCreateFolder: _createFolder,
            onRename: _renameFileOrFolder,
            onImportFile: _importFile,
            onFileTap: (path) {
              final ext = path.contains('.')
                  ? path.split('.').last.toLowerCase()
                  : '';
              final isUnsupportedBinary = [
                'zip',
                'tar',
                'gz',
                'exe',
                'dll',
                'so',
                'dylib',
                'db',
                'sqlite',
              ].contains(ext);
              if (!isUnsupportedBinary) _openFile(path);
            },
            onDelete: (path) {
              try {
                if (Directory(path).existsSync()) {
                  Directory(path).deleteSync(recursive: true);
                } else {
                  File(path).deleteSync();
                }
                _openTabs.removeWhere((t) => t.filePath == path);
                if (_openTabs.isEmpty) {
                  Navigator.pushReplacement(
                    context,
                    MaterialPageRoute(builder: (_) => const WelcomeScreen()),
                  );
                }
                setState(() => _explorerVersion++);
              } catch (e) {
                ScaffoldMessenger.of(
                  context,
                ).showSnackBar(SnackBar(content: Text('Gagal: $e')));
              }
            },
          ),
        );
        break;
      case ActivityType.search:
        panel = SearchPanel(
          rootPath: widget.project.path,
          onMatchTap: (filePath, line) => _openFile(filePath, lineNumber: line),
        );
        break;
      case ActivityType.chat:
        panel = ClassroomPanel(projectPath: widget.project.path);
        break;
      default:
        panel = Container(
          color: const Color(0xFF252526),
          child: const Center(
            child: Text(
              'Segera hadir',
              style: TextStyle(color: Colors.white30, fontSize: 12),
            ),
          ),
        );
    }

    return Row(
      children: [
        SizedBox(width: 240, child: panel),
        Container(width: 1, color: const Color(0xFF3C3C3C)),
      ],
    );
  }

  /// Empty state when no file is open.
  Widget _buildEditorOrViewer(EditorTab tab) {
    final lowerPath = tab.filePath.toLowerCase();
    
    final isSpreadsheet = lowerPath.endsWith('.csv') || lowerPath.endsWith('.xls') || lowerPath.endsWith('.xlsx');
    if (isSpreadsheet) {
      return SpreadsheetViewerWidget(filePath: tab.filePath);
    }
    
    final isPdf = lowerPath.endsWith('.pdf');
    if (isPdf) {
      return PdfViewerWidget(filePath: tab.filePath);
    }

    final isImage =
        lowerPath.endsWith('.png') ||
        lowerPath.endsWith('.jpg') ||
        lowerPath.endsWith('.jpeg') ||
        lowerPath.endsWith('.gif') ||
        lowerPath.endsWith('.webp') ||
        lowerPath.endsWith('.bmp');

    if (isImage) {
      return Container(
        color: const Color(0xFF1E1E1E),
        alignment: Alignment.center,
        child: InteractiveViewer(
          minScale: 0.1,
          maxScale: 10.0,
          child: Image.file(
            File(tab.filePath),
            errorBuilder: (context, error, stackTrace) => const Text(
              'Failed to load image',
              style: TextStyle(color: Colors.red),
            ),
          ),
        ),
      );
    }

    return CodeEditor(
      key: ValueKey('${tab.filePath}-$_targetLineNumber-$_localSearchQuery'),
      tab: tab,
      initialLineNumber: _targetLineNumber,
      searchQuery: _localSearchQuery,
      onChanged: (content, selStart, selEnd) {
        tab.content = content;
        if (_classroomService.isHost) {
          _classroomService.broadcastLiveCode(content, selectionStart: selStart, selectionEnd: selEnd);
        } else if (_classroomService.isBroadcastingToHost) {
          _classroomService.sendStudentLiveCode(content, selectionStart: selStart, selectionEnd: selEnd);
        }
        setState(() {});
      },
      onSave: (path, content) => setState(() {}),
    );
  }

  Widget _buildEmptyEditor() {
    return Container(
      color: const Color(0xFF1E1E1E),
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            HugeIcon(
              icon: HugeIcons.strokeRoundedSourceCode,
              size: 48,
              color: Colors.white.withOpacity(0.08),
            ),
            const SizedBox(height: 12),
            Text(
              'Tidak ada file yang dibuka',
              style: TextStyle(
                color: Colors.white.withOpacity(0.25),
                fontSize: 13,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMinimizedLiveCode(bool isMobile) {
    return InkWell(
      onTap: () => setState(() => _isLiveCodeMinimized = false),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xFF2D2D2D),
          border: Border(
            left: isMobile ? BorderSide.none : const BorderSide(color: Color(0xFF3C3C3C)),
            top: isMobile ? const BorderSide(color: Color(0xFF3C3C3C)) : BorderSide.none,
          ),
        ),
        height: isMobile ? 35 : null,
        width: isMobile ? null : 40,
        child: isMobile
            ? Row(
                children: [
                  const SizedBox(width: 12),
                  HugeIcon(icon: HugeIcons.strokeRoundedLaptopProgramming, size: 16, color: Colors.blueAccent),
                  const SizedBox(width: 8),
                  const Text("Live Code Guru", style: TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold)),
                  const Spacer(),
                  const Icon(Icons.expand_less, size: 16, color: Colors.white54),
                  const SizedBox(width: 12),
                ],
              )
            : Column(
                children: [
                  const SizedBox(height: 12),
                  HugeIcon(icon: HugeIcons.strokeRoundedLaptopProgramming, size: 16, color: Colors.blueAccent),
                  const Spacer(),
                  const Icon(Icons.chevron_left, size: 16, color: Colors.white54),
                  const SizedBox(height: 12),
                ],
              ),
      ),
    );
  }

  void _scrollIosTerminalToBottom() {
    Future.delayed(const Duration(milliseconds: 50), () {
      if (_iosTerminalScrollController.hasClients) {
        _iosTerminalScrollController.animateTo(
          _iosTerminalScrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _handleIosTerminalCommand(String input) async {
    final cmd = input.trim();
    if (cmd.isEmpty) return;

    _iosTerminalInputController.clear();
    setState(() {
      _iosTerminalLines.add('>_ $cmd');
    });

    final parts = cmd.split(' ');
    final baseCmd = parts[0].toLowerCase();
    final args = parts.sublist(1);

    final forbiddenKeywords = ['rm', 'rf', 'format', 'mkfs', 'dd', 'shred', 'wipe', 'fdisk', 'parted', 'chmod', 'chown', 'sudo', 'su', 'del', 'rd', 'erase'];
    if (forbiddenKeywords.contains(baseCmd) || cmd.contains('rm ') || cmd.contains('del ')) {
      setState(() {
        _iosTerminalLines.add('⚠ Perintah ini dilarang demi keamanan.');
      });
      _scrollIosTerminalToBottom();
      return;
    }

    switch (baseCmd) {
      case 'clear':
      case 'cls':
        setState(() => _iosTerminalLines.clear());
        break;
      case 'help':
        setState(() {
          _iosTerminalLines.addAll([
            '╭─ Perintah yang tersedia ──────────╮',
            '│  help        Bantuan               │',
            '│  pwd         Direktori saat ini     │',
            '│  ls          Daftar berkas          │',
            '│  cd [dir]    Pindah direktori       │',
            '│  cat [file]  Baca isi berkas        │',
            '│  run [file]  Jalankan program RPL   │',
            '│  clear       Bersihkan layar        │',
            '╰─────────────────────────────────────╯',
          ]);
        });
        break;
      case 'pwd':
        setState(() => _iosTerminalLines.add(_iosTerminalCwd));
        break;
      case 'ls':
        try {
          final dir = Directory(_iosTerminalCwd);
          if (dir.existsSync()) {
            final contents = dir.listSync();
            if (contents.isEmpty) {
              setState(() => _iosTerminalLines.add('(kosong)'));
            } else {
              for (var entity in contents) {
                final isDir = entity is Directory;
                final name = entity.path.split(Platform.pathSeparator).last;
                setState(() {
                  _iosTerminalLines.add('${isDir ? "📁" : "📄"} $name');
                });
              }
            }
          }
        } catch (e) {
          setState(() => _iosTerminalLines.add('⚠ $e'));
        }
        break;
      case 'cd':
        if (args.isEmpty) {
          setState(() => _iosTerminalCwd = widget.project.path);
        } else {
          final target = args.join(' ');
          String newPath;
          if (target == '..') {
            final parent = Directory(_iosTerminalCwd).parent.path;
            newPath = parent.startsWith(widget.project.path) ? parent : widget.project.path;
          } else {
            newPath = Directory('$_iosTerminalCwd${Platform.pathSeparator}$target').path;
          }
          final dir = Directory(newPath);
          if (dir.existsSync()) {
            setState(() => _iosTerminalCwd = newPath);
          } else {
            setState(() => _iosTerminalLines.add('⚠ Folder "$target" tidak ditemukan.'));
          }
        }
        break;
      case 'cat':
        if (args.isEmpty) {
          setState(() => _iosTerminalLines.add('Gunakan: cat [nama_file]'));
        } else {
          final fileName = args.join(' ');
          final filePath = '$_iosTerminalCwd${Platform.pathSeparator}$fileName';
          final file = File(filePath);
          if (file.existsSync()) {
            try {
              final content = file.readAsStringSync();
              setState(() => _iosTerminalLines.addAll(content.split('\n')));
            } catch (e) {
              setState(() => _iosTerminalLines.add('⚠ $e'));
            }
          } else {
            setState(() => _iosTerminalLines.add('⚠ File "$fileName" tidak ditemukan.'));
          }
        }
        break;
      case 'run':
      case 'rpl':
        if (args.isEmpty) {
          setState(() => _iosTerminalLines.add('Gunakan: run [nama_file.rpl]'));
        } else {
          final fileName = args.join(' ');
          final filePath = '$_iosTerminalCwd${Platform.pathSeparator}$fileName';
          final file = File(filePath);
          if (file.existsSync()) {
            setState(() => _iosTerminalLines.add('⏳ Menjalankan $fileName...'));
            try {
              final content = file.readAsStringSync();
              final output = await runCode(code: content);
              setState(() => _iosTerminalLines.addAll(output.split('\n')));
            } catch (e) {
              setState(() => _iosTerminalLines.add('⚠ $e'));
            }
          } else {
            setState(() => _iosTerminalLines.add('⚠ File "$fileName" tidak ditemukan.'));
          }
        }
        break;
      default:
        setState(() {
          _iosTerminalLines.add('⚠ "$baseCmd" tidak dikenali. Ketik "help".');
        });
        break;
    }
    _scrollIosTerminalToBottom();
  }

  /// Terminal panel — always present, can be minimized.
  Widget _buildTerminal(SettingsState settings) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      curve: Curves.easeOut,
      height: _isTerminalMinimized ? 29 : settings.terminalHeight,
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        border: Border(top: BorderSide(color: Color(0xFF3C3C3C))),
      ),
      child: Column(
        children: [
          // Terminal Header
          InkWell(
            onTap: () =>
                setState(() => _isTerminalMinimized = !_isTerminalMinimized),
            child: Container(
              height: 28,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Row(
                children: [
                  Icon(
                    _isTerminalMinimized
                        ? Icons.expand_less
                        : Icons.expand_more,
                    size: 14,
                    color: Colors.white38,
                  ),
                  const SizedBox(width: 6),
                  const Text(
                    'TERMINAL',
                    style: TextStyle(
                      color: Colors.white54,
                      fontWeight: FontWeight.w600,
                      fontSize: 11,
                      letterSpacing: 0.5,
                    ),
                  ),
                  const Spacer(),
                  if (!_isTerminalMinimized) ...[
                    GestureDetector(
                      onTap: () {
                        String textToCopy = '';
                        if (Platform.isIOS) {
                          textToCopy = _iosTerminalLines.join('\n');
                        } else {
                          textToCopy = _terminal.buffer.getText();
                        }
                        if (textToCopy.isNotEmpty) {
                          Clipboard.setData(ClipboardData(text: textToCopy));
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(
                              content: Text('Terminal output disalin ke clipboard'),
                              backgroundColor: Color(0xFF2568E7),
                              duration: Duration(seconds: 2),
                            ),
                          );
                        }
                      },
                      child: const HugeIcon(
                        icon: HugeIcons.strokeRoundedCopy01,
                        size: 14,
                        color: Colors.white30,
                      ),
                    ),
                    const SizedBox(width: 12),
                    GestureDetector(
                      onTap: () {
                        if (Platform.isIOS) {
                          setState(() => _iosTerminalLines.clear());
                        } else {
                          _terminal.buffer.clear();
                          _terminal.buffer.setCursor(0, 0);
                        }
                      },
                      child: const HugeIcon(
                        icon: HugeIcons.strokeRoundedDelete02,
                        size: 14,
                        color: Colors.white30,
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
          // Terminal Body
          if (!_isTerminalMinimized) ...[
            if (Platform.isIOS)
              Expanded(
                child: GestureDetector(
                  onTap: () => _terminalFocusNode.requestFocus(),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12),
                    child: ListView.builder(
                      controller: _iosTerminalScrollController,
                      itemCount: _iosTerminalLines.length,
                      itemBuilder: (context, idx) {
                        final line = _iosTerminalLines[idx];
                        final isPrompt = line.startsWith('>_');
                        return Padding(
                          padding: const EdgeInsets.symmetric(vertical: 1.0),
                          child: SelectableText(
                            line,
                            style: TextStyle(
                              color: isPrompt ? const Color(0xFF4EC9B0) : Colors.white70,
                              fontFamily: 'monospace',
                              fontSize: settings.terminalFontSize,
                              fontWeight: isPrompt ? FontWeight.w600 : FontWeight.normal,
                            ),
                          ),
                        );
                      },
                    ),
                  ),
                ),
              )
            else
              Expanded(
                child: Container(
                  color: Colors.black,
                  child: TerminalView(
                    _terminal,
                    controller: _terminalController,
                    focusNode: _terminalFocusNode,
                    textStyle: TerminalStyle(
                      fontFamily: 'monospace',
                      fontSize: settings.terminalFontSize,
                    ),
                  ),
                ),
              ),
            if (Platform.isIOS)
              Container(
                height: 28,
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: Row(
                  children: [
                    const Text(
                      '>_ ',
                      style: TextStyle(
                        color: Color(0xFF4EC9B0),
                        fontFamily: 'monospace',
                        fontSize: 12,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    Expanded(
                      child: Theme(
                        data: Theme.of(context).copyWith(
                          inputDecorationTheme: const InputDecorationTheme(
                            border: InputBorder.none,
                            enabledBorder: InputBorder.none,
                            focusedBorder: InputBorder.none,
                            filled: false,
                          ),
                        ),
                        child: TextField(
                          controller: _iosTerminalInputController,
                          focusNode: _terminalFocusNode,
                          onSubmitted: _handleIosTerminalCommand,
                          style: const TextStyle(
                            color: Colors.white,
                            fontFamily: 'monospace',
                            fontSize: 12,
                          ),
                          decoration: const InputDecoration(
                            isDense: true,
                            contentPadding: EdgeInsets.symmetric(vertical: 4),
                            border: InputBorder.none,
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
          ],
        ],
      ),
    );
  }

  Widget _buildKeyboardToolbar() {
    final symbols = [
      '<',
      '>',
      '{',
      '}',
      '[',
      ']',
      '(',
      ')',
      '=',
      '"',
      "'",
      '`',
      ';',
      ':',
      ',',
      '.',
    ];
    return Container(
      height: 40,
      color: const Color(0xFF2D2D2D),
      child: ListView.builder(
        scrollDirection: Axis.horizontal,
        itemCount: symbols.length,
        itemBuilder: (context, index) {
          final symbol = symbols[index];
          return InkWell(
            onTap: () {
              if (_terminalFocusNode.hasFocus) {
                _pty?.write(const Utf8Encoder().convert(symbol));
              } else {
                KeyboardEventNotifier.symbolStream.add(symbol);
              }
            },
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              alignment: Alignment.center,
              child: Text(
                symbol,
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 16,
                  fontFamily: 'monospace',
                ),
              ),
            ),
          );
        },
      ),
    );
  }
}

/// Small title bar button with optional active state.
class _TitleBarButton extends StatelessWidget {
  final dynamic icon;
  final String tooltip;
  final bool isActive;
  final Color? color;
  final VoidCallback? onPressed;

  const _TitleBarButton({
    required this.icon,
    required this.tooltip,
    this.isActive = false,
    this.color,
    this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: icon is IconData
          ? Icon(
              icon,
              size: 15,
              color:
                  color ??
                  (isActive ? const Color(0xFF2568E7) : Colors.white38),
            )
          : HugeIcon(
              icon: icon,
              size: 15,
              color:
                  color ??
                  (isActive ? const Color(0xFF2568E7) : Colors.white38),
            ),
      padding: EdgeInsets.zero,
      constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
      tooltip: tooltip,
      onPressed: onPressed,
    );
  }
}

class _LiveCursorPainter extends CustomPainter {
  final String text;
  final int selectionStart;
  final int selectionEnd;
  final String hostName;
  final TextStyle textStyle;

  _LiveCursorPainter({
    required this.text,
    required this.selectionStart,
    required this.selectionEnd,
    required this.hostName,
    required this.textStyle,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (text.isEmpty) return;

    final paddingOffset = const Offset(4, 4); // Sesuai dengan padding HighlightView(EdgeInsets.all(4))

    final textPainter = TextPainter(
      text: TextSpan(text: text, style: textStyle),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout(minWidth: 0, maxWidth: size.width);

    final start = selectionStart <= selectionEnd ? selectionStart : selectionEnd;
    final end = selectionStart <= selectionEnd ? selectionEnd : selectionStart;

    // 1. Draw Selection Boxes
    if (start != end) {
      final selectionPaint = Paint()..color = Colors.blueAccent.withAlpha(80);
      try {
        final boxes = textPainter.getBoxesForSelection(
          TextSelection(baseOffset: start, extentOffset: end),
        );
        
        for (final box in boxes) {
          canvas.drawRect(
            box.toRect().shift(paddingOffset),
            selectionPaint,
          );
        }
      } catch (e) {
        // Abaikan error layout (mis. index di luar teks)
      }
    }

    // 2. Draw Cursor Line
    final cursorPaint = Paint()
      ..color = Colors.orangeAccent
      ..strokeWidth = 2.0;

    Offset caretOffset;
    try {
      caretOffset = textPainter.getOffsetForCaret(
        TextPosition(offset: selectionEnd),
        Rect.zero,
      );
    } catch (e) {
      caretOffset = Offset.zero;
    }
    
    final finalCaretOffset = caretOffset + paddingOffset;

    canvas.drawLine(
      finalCaretOffset,
      finalCaretOffset + Offset(0, textStyle.fontSize! * (textStyle.height ?? 1.0)),
      cursorPaint,
    );

    // 3. Draw Host Name Label
    final labelPainter = TextPainter(
      text: TextSpan(
        text: ' $hostName ',
        style: const TextStyle(
          color: Colors.black,
          backgroundColor: Colors.orangeAccent,
          fontSize: 10,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    );
    labelPainter.layout();
    
    // Position label slightly di atas cursor
    labelPainter.paint(
      canvas,
      finalCaretOffset - const Offset(0, 14),
    );
  }

  @override
  bool shouldRepaint(covariant _LiveCursorPainter oldDelegate) {
    return text != oldDelegate.text ||
        selectionStart != oldDelegate.selectionStart ||
        selectionEnd != oldDelegate.selectionEnd ||
        hostName != oldDelegate.hostName;
  }
}

