import 'dart:io';
import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';
import '../../models/project.dart';
import '../../services/project_service.dart';
import '../editor/editor_tab.dart';
import '../editor/code_editor.dart';
import '../explorer/file_explorer.dart';
import 'welcome_screen.dart';
import '../../src/rust/api/simple.dart';
import 'activity_bar.dart';
import 'search_panel.dart';
import '../browser/browser_workspace.dart';
import '../database/database_workspace.dart';

enum WorkspaceType { editor, browser, database }

class ProjectScreen extends StatefulWidget {
  final Project project;
  const ProjectScreen({super.key, required this.project});
  @override
  State<ProjectScreen> createState() => _ProjectScreenState();
}

class _ProjectScreenState extends State<ProjectScreen> {
  late List<EditorTab> _openTabs;
  int _activeTabIndex = 0;
  ActivityType? _activeActivity = ActivityType.explorer;
  bool _isTerminalMinimized = false;
  int _explorerVersion = 0;
  int? _targetLineNumber;

  WorkspaceType _activeWorkspace = WorkspaceType.editor;

  bool _showLocalSearch = false;
  String _localSearchQuery = '';
  final TextEditingController _localSearchController = TextEditingController();

  late String _terminalCwd;
  final List<String> _terminalLines = [];
  final TextEditingController _terminalInputController =
      TextEditingController();
  final ScrollController _terminalScrollController = ScrollController();
  final FocusNode _terminalFocusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _terminalCwd = widget.project.path;
    _terminalLines.add('Selamat datang di RPL Studio Terminal!');
    _terminalLines.add('Ketik "help" untuk melihat daftar perintah.');
    _terminalLines.add('');

    final mainFile = _findMainFile();
    _openTabs = [
      EditorTab(
        filePath: mainFile.path,
        fileName: mainFile.path.split(Platform.pathSeparator).last,
        content: _readFile(mainFile.path),
      ),
    ];
    ProjectService.touchProject(widget.project);
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
    _terminalInputController.dispose();
    _terminalScrollController.dispose();
    _terminalFocusNode.dispose();
    _localSearchController.dispose();
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
                backgroundColor: const Color(0xFF007ACC),
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
                borderSide: BorderSide(color: Color(0xFF007ACC)),
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
              backgroundColor: const Color(0xFF007ACC),
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
                borderSide: BorderSide(color: Color(0xFF007ACC)),
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
              backgroundColor: const Color(0xFF007ACC),
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
      final result = await FilePicker.pickFiles(allowMultiple: false);
      if (result != null && result.files.single.name != null) {
        final fileName = result.files.single.name;
        final destinationPath =
            '$targetDirectoryPath${Platform.pathSeparator}$fileName';

        if (result.files.single.path != null) {
          final sourceFile = File(result.files.single.path!);
          await sourceFile.copy(destinationPath);
        } else if (result.files.single.bytes != null) {
          await File(destinationPath).writeAsBytes(result.files.single.bytes!);
        }

        setState(() => _explorerVersion++);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text('Berhasil mengimpor $fileName'),
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

  void _scrollToTerminalBottom() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_terminalScrollController.hasClients) {
        _terminalScrollController.animateTo(
          _terminalScrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 150),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _handleTerminalCommand(String input) async {
    final cmd = input.trim();
    if (cmd.isEmpty) return;

    _terminalInputController.clear();
    setState(() {
      _terminalLines.add('>_ $cmd');
    });

    final parts = cmd.split(' ');
    final baseCmd = parts[0].toLowerCase();
    final args = parts.sublist(1);

    // Filter destructive commands
    final forbiddenKeywords = [
      'rm',
      'rf',
      'format',
      'mkfs',
      'dd',
      'shred',
      'wipe',
      'fdisk',
      'parted',
      'chmod',
      'chown',
      'sudo',
      'su',
      'del',
      'rd',
      'erase',
    ];
    if (forbiddenKeywords.contains(baseCmd) ||
        cmd.contains('rm ') ||
        cmd.contains('del ')) {
      setState(() {
        _terminalLines.add('⚠ Perintah ini dilarang demi keamanan.');
      });
      _scrollToTerminalBottom();
      return;
    }

    switch (baseCmd) {
      case 'clear':
      case 'cls':
        setState(() => _terminalLines.clear());
        break;

      case 'help':
        setState(() {
          _terminalLines.addAll([
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
        setState(() => _terminalLines.add(_terminalCwd));
        break;

      case 'ls':
        try {
          final dir = Directory(_terminalCwd);
          if (dir.existsSync()) {
            final contents = dir.listSync();
            if (contents.isEmpty) {
              setState(() => _terminalLines.add('(kosong)'));
            } else {
              for (var entity in contents) {
                final isDir = entity is Directory;
                final name = entity.path.split(Platform.pathSeparator).last;
                setState(() {
                  _terminalLines.add('${isDir ? "📁" : "📄"} $name');
                });
              }
            }
          }
        } catch (e) {
          setState(() => _terminalLines.add('⚠ $e'));
        }
        break;

      case 'cd':
        if (args.isEmpty) {
          setState(() => _terminalCwd = widget.project.path);
        } else {
          final target = args.join(' ');
          String newPath;
          if (target == '..') {
            final parent = Directory(_terminalCwd).parent.path;
            newPath = parent.startsWith(widget.project.path)
                ? parent
                : widget.project.path;
          } else {
            newPath = Directory(
              '$_terminalCwd${Platform.pathSeparator}$target',
            ).path;
          }

          final dir = Directory(newPath);
          if (dir.existsSync()) {
            setState(() => _terminalCwd = newPath);
          } else {
            setState(
              () => _terminalLines.add('⚠ Folder "$target" tidak ditemukan.'),
            );
          }
        }
        break;

      case 'cat':
        if (args.isEmpty) {
          setState(() => _terminalLines.add('Gunakan: cat [nama_file]'));
        } else {
          final fileName = args.join(' ');
          final filePath = '$_terminalCwd${Platform.pathSeparator}$fileName';
          final file = File(filePath);
          if (file.existsSync()) {
            try {
              final content = file.readAsStringSync();
              setState(() => _terminalLines.addAll(content.split('\n')));
            } catch (e) {
              setState(() => _terminalLines.add('⚠ $e'));
            }
          } else {
            setState(
              () => _terminalLines.add('⚠ File "$fileName" tidak ditemukan.'),
            );
          }
        }
        break;

      case 'run':
      case 'rpl':
        if (args.isEmpty) {
          setState(() => _terminalLines.add('Gunakan: run [nama_file.rpl]'));
        } else {
          final fileName = args.join(' ');
          final filePath = '$_terminalCwd${Platform.pathSeparator}$fileName';
          final file = File(filePath);
          if (file.existsSync()) {
            setState(() => _terminalLines.add('⏳ Menjalankan $fileName...'));
            try {
              final content = file.readAsStringSync();
              final output = await runCode(code: content);
              if (!mounted) return;
              setState(() => _terminalLines.addAll(output.split('\n')));
            } catch (e) {
              if (!mounted) return;
              setState(() => _terminalLines.add('⚠ $e'));
            }
          } else {
            setState(
              () => _terminalLines.add('⚠ File "$fileName" tidak ditemukan.'),
            );
          }
        }
        break;

      default:
        setState(() {
          _terminalLines.add('⚠ "$baseCmd" tidak dikenali. Ketik "help".');
        });
        break;
    }

    _scrollToTerminalBottom();
  }

  @override
  Widget build(BuildContext context) {
    final mediaQuery = MediaQuery.of(context);
    final isMobile = mediaQuery.size.width < 600;
    final isBrowser = _activeWorkspace == WorkspaceType.browser;
    final isDatabase = _activeWorkspace == WorkspaceType.database;
    final isKeyboardOpen = mediaQuery.viewInsets.bottom > 0;
    final isTerminalFocused = _terminalFocusNode.hasFocus;

    // Determine if any input is focused so we can show the toolbar
    final showToolbar = isMobile && isKeyboardOpen;

    return Scaffold(
      backgroundColor: const Color(0xFF1E1E1E),
      bottomNavigationBar: showToolbar ? _buildKeyboardToolbar() : null,
      body: SafeArea(
        child: Column(
          children: [
            // ═══ Title Bar / Navbar ═══
            if (!isBrowser && !isDatabase) _buildTitleBar(),
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
                          } else {
                            _activeWorkspace = WorkspaceType.editor;
                          }
                        }
                      });
                    },
                  ),
                  Expanded(
                    child: IndexedStack(
                      index: isBrowser ? 1 : (isDatabase ? 2 : 0),
                      children: [
                        // Index 0: Editor & Terminal
                        Row(
                          children: [
                            // Side Panel (only in layout hierarchy on Desktop)
                            if (!isMobile) _buildSidePanel(),
                            // Editor + Terminal
                            Expanded(
                              child: Stack(
                                children: [
                                  Column(
                                    children: [
                                      // Tab Bar
                                      EditorTabBar(
                                        tabs: _openTabs,
                                        activeIndex: _activeTabIndex,
                                        onTap: (i) =>
                                            setState(() => _activeTabIndex = i),
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
                                      _buildTerminal(),
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
                          ],
                        ),
                        // Index 1: Browser Workspace
                        const BrowserWorkspace(),
                        // Index 2: Database Workspace
                        DatabaseWorkspace(projectPath: widget.project.path),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
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
          // Back button
          IconButton(
            icon: const Icon(
              Icons.arrow_back_ios_new,
              size: 13,
              color: Colors.white54,
            ),
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 28, minHeight: 28),
            tooltip: 'Kembali ke Welcome',
            onPressed: () => Navigator.pushReplacement(
              context,
              MaterialPageRoute(builder: (_) => const WelcomeScreen()),
            ),
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
          if (_openTabs.isNotEmpty) ...[
            ValueListenableBuilder<UndoHistoryValue>(
              valueListenable: _openTabs[_activeTabIndex].undoController,
              builder: (context, value, child) {
                return Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    _TitleBarButton(
                      icon: Icons.undo,
                      tooltip: 'Undo',
                      color: value.canUndo ? Colors.white70 : Colors.white24,
                      onPressed: value.canUndo
                          ? () =>
                                _openTabs[_activeTabIndex].undoController.undo()
                          : null,
                    ),
                    _TitleBarButton(
                      icon: Icons.redo,
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
              icon: Icons.search,
              tooltip: 'Cari di file ini',
              isActive: _showLocalSearch,
              onPressed: () {
                setState(() {
                  _showLocalSearch = !_showLocalSearch;
                  if (!_showLocalSearch) {
                    _localSearchQuery = '';
                    _localSearchController.clear();
                  }
                });
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
                _terminalLines.add(
                  '>_ run ${_openTabs[_activeTabIndex].fileName}',
                );
                _terminalLines.add(
                  '⏳ Menjalankan ${_openTabs[_activeTabIndex].fileName}...',
                );
              });
              final content = _openTabs[_activeTabIndex].content;
              final result = await runCode(code: content);
              if (!mounted) return;
              setState(() => _terminalLines.addAll(result.split('\n')));
              _scrollToTerminalBottom();
            },
          ),
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
            prefixIcon: const Icon(
              Icons.search,
              size: 14,
              color: Colors.white38,
            ),
            prefixIconConstraints: const BoxConstraints(minWidth: 28),
            suffixIcon: IconButton(
              icon: const Icon(Icons.close, size: 12, color: Colors.white54),
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
        panel = FileExplorer(
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
              'pdf',
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
        );
        break;
      case ActivityType.search:
        panel = SearchPanel(
          rootPath: widget.project.path,
          onMatchTap: (filePath, line) => _openFile(filePath, lineNumber: line),
        );
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
      onChanged: () => setState(() {}),
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
            Icon(Icons.code, size: 48, color: Colors.white.withOpacity(0.08)),
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

  /// Terminal panel — always present, can be minimized.
  Widget _buildTerminal() {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 200),
      curve: Curves.easeOut,
      height: _isTerminalMinimized ? 29 : 180,
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
                        ? Icons.keyboard_arrow_up
                        : Icons.keyboard_arrow_down,
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
                  if (!_isTerminalMinimized)
                    GestureDetector(
                      onTap: () => setState(() => _terminalLines.clear()),
                      child: const Icon(
                        Icons.delete_outline,
                        size: 14,
                        color: Colors.white30,
                      ),
                    ),
                ],
              ),
            ),
          ),
          // Terminal Body
          if (!_isTerminalMinimized) ...[
            Expanded(
              child: GestureDetector(
                onTap: () => _terminalFocusNode.requestFocus(),
                child: Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12),
                  child: ListView.builder(
                    controller: _terminalScrollController,
                    itemCount: _terminalLines.length,
                    itemBuilder: (context, idx) {
                      final line = _terminalLines[idx];
                      final isPrompt = line.startsWith('>_');
                      return Padding(
                        padding: const EdgeInsets.symmetric(vertical: 1.0),
                        child: Text(
                          line,
                          style: TextStyle(
                            color: isPrompt
                                ? const Color(0xFF4EC9B0)
                                : Colors.white70,
                            fontFamily: 'monospace',
                            fontSize: 12,
                            fontWeight: isPrompt
                                ? FontWeight.w600
                                : FontWeight.normal,
                          ),
                        ),
                      );
                    },
                  ),
                ),
              ),
            ),
            // Input row
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
                        controller: _terminalInputController,
                        focusNode: _terminalFocusNode,
                        onSubmitted: _handleTerminalCommand,
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
                final selection = _terminalInputController.selection;
                if (selection.baseOffset >= 0) {
                  final text = _terminalInputController.text;
                  final newText = text.replaceRange(
                    selection.start,
                    selection.end,
                    symbol,
                  );
                  _terminalInputController.value = _terminalInputController
                      .value
                      .copyWith(
                        text: newText,
                        selection: TextSelection.collapsed(
                          offset: selection.start + symbol.length,
                        ),
                      );
                } else {
                  _terminalInputController.text += symbol;
                }
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
  final IconData icon;
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
      icon: Icon(
        icon,
        size: 15,
        color: color ?? (isActive ? const Color(0xFF007ACC) : Colors.white38),
      ),
      padding: EdgeInsets.zero,
      constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
      tooltip: tooltip,
      onPressed: onPressed,
    );
  }
}
