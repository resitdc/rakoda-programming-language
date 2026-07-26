import 'dart:io';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
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
import '../http/http_workspace.dart';
import '../chat/chat_panel.dart';
import 'pdf_viewer_widget.dart';
import 'spreadsheet_viewer_widget.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../settings/settings_provider.dart';
import 'package:xterm/xterm.dart';
import 'package:flutter_pty/flutter_pty.dart';
import 'package:desktop_drop/desktop_drop.dart';
import 'dart:convert';

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
  bool _isTerminalMinimized = false;
  int _explorerVersion = 0;
  int? _targetLineNumber;

  WorkspaceType _activeWorkspace = WorkspaceType.editor;

  bool _showLocalSearch = false;
  String _localSearchQuery = '';
  final TextEditingController _localSearchController = TextEditingController();

  late final Terminal _terminal;
  Pty? _pty;
  final FocusNode _terminalFocusNode = FocusNode();

  @override
  void initState() {
    super.initState();
    _terminal = Terminal();

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
          });

      _terminal.onOutput = (data) {
        _pty?.write(const Utf8Encoder().convert(data));
      };

      _terminal.onResize = (w, h, pw, ph) {
        _pty?.resize(h, w);
      };
    } catch (e) {
      _terminal.write('\x1b[31mTerminal Error: $e\x1b[0m\r\n');
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
    final mediaQuery = MediaQuery.of(context);
    final isMobile = mediaQuery.size.width < 600;
    final isBrowser = _activeWorkspace == WorkspaceType.browser;
    final isDatabase = _activeWorkspace == WorkspaceType.database;
    final isHttp = _activeWorkspace == WorkspaceType.http;
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
            if (!isBrowser && !isDatabase && !isHttp) _buildTitleBar(),
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
              _terminal.write(
                '\r\n>_ run ${_openTabs[_activeTabIndex].fileName}\r\n',
              );
              final content = _openTabs[_activeTabIndex].content;
              final result = await runCode(code: content);
              if (!mounted) return;
              _terminal.write(result.replaceAll('\n', '\r\n') + '\r\n');
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
        panel = const ChatPanel();
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
                  if (!_isTerminalMinimized)
                    GestureDetector(
                      onTap: () {
                        _terminal.buffer.clear();
                        _terminal.buffer.setCursor(0, 0);
                      },
                      child: HugeIcon(
                        icon: HugeIcons.strokeRoundedDelete02,
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
              child: Container(
                color: Colors.black,
                child: TerminalView(
                  _terminal,
                  focusNode: _terminalFocusNode,
                  textStyle: const TerminalStyle(
                    fontFamily: 'monospace',
                    fontSize: 13,
                  ),
                ),
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
