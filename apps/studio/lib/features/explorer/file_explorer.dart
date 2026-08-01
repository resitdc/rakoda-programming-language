import 'dart:io';
import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hugeicons/hugeicons.dart';
import '../../shared/file_icon_helper.dart';
import '../settings/settings_provider.dart';

/// File/folder node untuk tree view.
class FileNode {
  final String name;
  final String path;
  final bool isDirectory;
  final List<FileNode> children;
  bool isExpanded;

  FileNode({
    required this.name,
    required this.path,
    required this.isDirectory,
    List<FileNode>? children,
    this.isExpanded = false,
  }) : children = children ?? [];
}

/// Widget tree view untuk project explorer.
class FileExplorer extends ConsumerStatefulWidget {
  final String rootPath;
  final int refreshTrigger;
  final void Function(String path)? onFileTap;
  final void Function(String path, String oldName, String newName)? onRename;
  final void Function(String path)? onDelete;
  final void Function(String parentPath)? onCreateFile;
  final void Function(String parentPath)? onCreateFolder;
  final void Function(String parentPath)? onImportFile;

  const FileExplorer({
    super.key,
    required this.rootPath,
    this.refreshTrigger = 0,
    this.onFileTap,
    this.onRename,
    this.onDelete,
    this.onCreateFile,
    this.onCreateFolder,
    this.onImportFile,
  });

  @override
  ConsumerState<FileExplorer> createState() => _FileExplorerState();
}

class _FileExplorerState extends ConsumerState<FileExplorer> {
  late List<FileNode> _roots;
  bool _loading = true;
  String? _selectedPath;
  final Set<String> _expandedPaths = {};
  
  StreamSubscription<FileSystemEvent>? _watchSubscription;
  Timer? _debounceTimer;

  @override
  void initState() {
    super.initState();
    _loadRoot();
    _startWatching();
  }

  @override
  void didUpdateWidget(FileExplorer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.rootPath != widget.rootPath) {
      _stopWatching();
      _loadRoot();
      _startWatching();
    } else if (oldWidget.refreshTrigger != widget.refreshTrigger) {
      _loadRoot();
    }
  }

  @override
  void dispose() {
    _stopWatching();
    _debounceTimer?.cancel();
    super.dispose();
  }

  void _startWatching() {
    if (ref.read(settingsProvider).isLowEndMode) return;

    try {
      final dir = Directory(widget.rootPath);
      if (dir.existsSync()) {
        _watchSubscription = dir.watch(recursive: true).listen((event) {
          _debounceTimer?.cancel();
          _debounceTimer = Timer(const Duration(milliseconds: 300), () {
            if (mounted) _loadRoot();
          });
        });
      }
    } catch (e) {
      // Ingore if filesystem doesn't support watching
    }
  }

  void _stopWatching() {
    _watchSubscription?.cancel();
    _watchSubscription = null;
  }

  void _loadRoot() {
    setState(() => _loading = true);
    _roots = _scanDirectory(widget.rootPath);
    setState(() => _loading = false);
  }

  List<FileNode> _scanDirectory(String path) {
    final dir = Directory(path);
    if (!dir.existsSync()) return [];

    final entries = dir.listSync()
      ..sort((a, b) {
        final aIsDir = a is Directory;
        final bIsDir = b is Directory;
        if (aIsDir && !bIsDir) return -1;
        if (!aIsDir && bIsDir) return 1;
        return a.path.compareTo(b.path);
      });

    return entries
        .map((entity) {
          final name = entity.path.split(Platform.pathSeparator).last;
          final isDirectory = entity is Directory;
          final isExpanded = isDirectory && _expandedPaths.contains(entity.path);
          
          List<FileNode> children = [];
          if (isExpanded) {
            children = _scanDirectory(entity.path);
          }

          return FileNode(
            name: name,
            path: entity.path,
            isDirectory: isDirectory,
            isExpanded: isExpanded,
            children: children,
          );
        })
        .where((node) => !node.name.startsWith('.'))
        .toList();
  }

  void _toggleExpand(FileNode node) {
    setState(() {
      if (_expandedPaths.contains(node.path)) {
        _expandedPaths.remove(node.path);
        node.isExpanded = false;
        node.children.clear();
      } else {
        _expandedPaths.add(node.path);
        node.isExpanded = true;
        node.children.addAll(_scanDirectory(node.path));
      }
    });
  }

  String _getTargetPath() {
    if (_selectedPath != null) {
      if (FileSystemEntity.isDirectorySync(_selectedPath!)) {
        return _selectedPath!;
      } else {
        return File(_selectedPath!).parent.path;
      }
    }
    return widget.rootPath;
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<SettingsState>(settingsProvider, (previous, next) {
      if (previous?.isLowEndMode != next.isLowEndMode) {
        if (next.isLowEndMode) {
          _stopWatching();
        } else {
          _startWatching();
        }
      }
    });

    if (_loading) {
      return const Center(
        child: SizedBox(
          width: 24,
          height: 24,
          child: CircularProgressIndicator(
            strokeWidth: 2,
            valueColor: AlwaysStoppedAnimation<Color>(Colors.white54),
          ),
        ),
      );
    }

    return Container(
      color: const Color(0xFF252526),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Header with actions
          Container(
            height: 32,
            padding: const EdgeInsets.only(left: 10, right: 4),
            child: Row(
              children: [
                Flexible(
                  child: Text(
                    'EXPLORER',
                    style: TextStyle(
                      color: Colors.white.withOpacity(0.6),
                      fontSize: 11,
                      fontWeight: FontWeight.w600,
                      letterSpacing: 0.5,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const Spacer(),
                if (widget.onCreateFile != null)
                  _ExplorerActionButton(
                    icon: Icons.note_add,
                    tooltip: 'New File',
                    onPressed: () => widget.onCreateFile!(_getTargetPath()),
                  ),
                if (widget.onCreateFolder != null)
                  _ExplorerActionButton(
                    icon: Icons.create_new_folder_outlined,
                    tooltip: 'New Folder',
                    onPressed: () => widget.onCreateFolder!(_getTargetPath()),
                  ),
                if (widget.onImportFile != null)
                  _ExplorerActionButton(
                    icon: Icons.file_upload_outlined,
                    tooltip: 'Import File',
                    onPressed: () => widget.onImportFile!(_getTargetPath()),
                  ),
                _ExplorerActionButton(
                  icon: Icons.refresh,
                  tooltip: 'Refresh',
                  onPressed: _loadRoot,
                ),
              ],
            ),
          ),
          // Tree
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(vertical: 2),
              children: _roots.map((node) => _buildNode(node, 0)).toList(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNode(FileNode node, int depth) {
    final isSelected = _selectedPath == node.path;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () {
            setState(() => _selectedPath = node.path);
            if (node.isDirectory) {
              _toggleExpand(node);
            } else {
              widget.onFileTap?.call(node.path);
            }
          },
          onLongPress: () => _showContextMenu(context, node),
          onSecondaryTapDown: (details) => _showDesktopContextMenu(context, node, details.globalPosition),
          hoverColor: const Color(0xFF2A2D2E),
          child: Container(
            color: isSelected ? const Color(0xFF37373D) : Colors.transparent,
            padding: EdgeInsets.only(
              left: 8.0 + depth * 14.0,
              right: 4,
              top: 4,
              bottom: 4,
            ),
            child: Row(
              children: [
                if (node.isDirectory)
                  Icon(
                    node.isExpanded ? Icons.expand_more : Icons.chevron_right,
                    size: 14,
                    color: Colors.white38,
                  )
                else
                  const SizedBox(width: 14),
                const SizedBox(width: 2),
                node.isDirectory 
                    ? Icon(
                        node.isExpanded ? Icons.folder_open : Icons.folder_outlined,
                        size: 15,
                        color: const Color(0xFFE2C08D),
                      )
                    : FileIconHelper.getFileIcon(node.name, size: 15),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    node.name,
                    style: TextStyle(
                      fontSize: 12,
                      color: isSelected ? Colors.white : Colors.white70,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        ),
        if (node.isExpanded)
          ...node.children.map((child) => _buildNode(child, depth + 1)),
      ],
    );
  }



  void _showDesktopContextMenu(BuildContext context, FileNode node, Offset position) async {
    final result = await showMenu<String>(
      context: context,
      popUpAnimationStyle: AnimationStyle(
        duration: Duration.zero,
        reverseDuration: Duration.zero,
      ),
      position: RelativeRect.fromLTRB(
        position.dx,
        position.dy,
        position.dx + 1,
        position.dy + 1,
      ),
      color: const Color(0xFF252526),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      items: [
        PopupMenuItem(
          value: 'rename',
          height: 36,
          child: Row(
            children: [
              HugeIcon(icon: HugeIcons.strokeRoundedEdit02, color: Colors.white70, size: 18),
              const SizedBox(width: 12),
              const Text('Rename', style: TextStyle(color: Colors.white70, fontSize: 13)),
            ],
          ),
        ),
        PopupMenuItem(
          value: 'delete',
          height: 36,
          child: Row(
            children: [
              HugeIcon(icon: HugeIcons.strokeRoundedDelete02, color: Color(0xFFFF6B6B), size: 18),
              const SizedBox(width: 12),
              const Text('Delete', style: TextStyle(color: Color(0xFFFF6B6B), fontSize: 13)),
            ],
          ),
        ),
      ],
    );

    if (result == 'rename') {
      _handleRename(node);
    } else if (result == 'delete') {
      _handleDelete(node);
    }
  }

  void _showContextMenu(BuildContext context, FileNode node) {
    showModalBottomSheet(
      context: context,
      backgroundColor: const Color(0xFF252526),
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
      ),
      builder: (_) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 8),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 32,
                height: 3,
                margin: const EdgeInsets.only(bottom: 8),
                decoration: BoxDecoration(
                  color: Colors.white24,
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              ListTile(
                leading: HugeIcon(icon: HugeIcons.strokeRoundedEdit02, color: Colors.white70, size: 20),
                title: const Text('Rename', style: TextStyle(color: Colors.white70, fontSize: 13)),
                dense: true,
                onTap: () {
                  Navigator.pop(context);
                  _handleRename(node);
                },
              ),
              ListTile(
                leading: HugeIcon(icon: HugeIcons.strokeRoundedDelete02, color: Color(0xFFFF6B6B), size: 20),
                title: const Text('Delete', style: TextStyle(color: Color(0xFFFF6B6B), fontSize: 13)),
                dense: true,
                onTap: () {
                  Navigator.pop(context);
                  _handleDelete(node);
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  void _handleRename(FileNode node) {
    final controller = TextEditingController(text: node.name);
    showDialog(
      context: context,
      builder: (_) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        title: const Text('Rename', style: TextStyle(color: Colors.white, fontSize: 14)),
        content: Theme(
          data: Theme.of(context).copyWith(
            inputDecorationTheme: const InputDecorationTheme(
              filled: true,
              fillColor: Color(0xFF1E1E1E),
              border: OutlineInputBorder(borderSide: BorderSide(color: Color(0xFF3C3C3C))),
              enabledBorder: OutlineInputBorder(borderSide: BorderSide(color: Color(0xFF3C3C3C))),
              focusedBorder: OutlineInputBorder(borderSide: BorderSide(color: Color(0xFF2568E7))),
            ),
          ),
          child: TextField(
            controller: controller,
            autofocus: true,
            style: const TextStyle(color: Colors.white, fontSize: 13),
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
              final newName = controller.text.trim();
              if (newName.isNotEmpty && newName != node.name) {
                widget.onRename?.call(node.path, node.name, newName);
              }
              Navigator.pop(context);
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF2568E7),
              foregroundColor: Colors.white,
            ),
            child: const Text('Rename'),
          ),
        ],
      ),
    );
  }

  void _handleDelete(FileNode node) {
    showDialog(
      context: context,
      builder: (_) => AlertDialog(
        backgroundColor: const Color(0xFF252526),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        title: const Text('Hapus', style: TextStyle(color: Colors.white, fontSize: 14)),
        content: Text(
          'Hapus "${node.name}"?',
          style: const TextStyle(color: Colors.white60, fontSize: 13),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            style: TextButton.styleFrom(foregroundColor: Colors.white54),
            child: const Text('Batal'),
          ),
          ElevatedButton(
            onPressed: () {
              widget.onDelete?.call(node.path);
              Navigator.pop(context);
            },
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF5A1D1D),
              foregroundColor: const Color(0xFFFF6B6B),
            ),
            child: const Text('Hapus'),
          ),
        ],
      ),
    );
  }
}

class _ExplorerActionButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;

  const _ExplorerActionButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon, size: 14, color: Colors.white54),
      tooltip: tooltip,
      onPressed: onPressed,
      padding: EdgeInsets.zero,
      constraints: const BoxConstraints(minWidth: 22, minHeight: 22),
    );
  }
}
