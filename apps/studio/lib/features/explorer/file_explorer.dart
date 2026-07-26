import 'dart:io';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';

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
class FileExplorer extends StatefulWidget {
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
  State<FileExplorer> createState() => _FileExplorerState();
}

class _FileExplorerState extends State<FileExplorer> {
  late List<FileNode> _roots;
  bool _loading = true;
  String? _selectedPath;
  final Set<String> _expandedPaths = {};

  @override
  void initState() {
    super.initState();
    _loadRoot();
  }

  @override
  void didUpdateWidget(FileExplorer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.rootPath != widget.rootPath || oldWidget.refreshTrigger != widget.refreshTrigger) {
      _loadRoot();
    }
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
    if (_loading) {
      return const Center(
        child: CircularProgressIndicator(color: Color(0xFF2568E7), strokeWidth: 2),
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
                Icon(
                  node.isDirectory
                      ? (node.isExpanded ? Icons.folder_open : Icons.folder_outlined)
                      : _fileIcon(node.name),
                  size: 15,
                  color: node.isDirectory
                      ? const Color(0xFFE2C08D)
                      : _fileIconColor(node.name),
                ),
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

  IconData _fileIcon(String name) {
    final lowerName = name.toLowerCase();
    if (lowerName.endsWith('.rpl')) return Icons.code;
    if (lowerName.endsWith('.html')) return Icons.public;
    if (lowerName.endsWith('.css')) return Icons.style;
    if (lowerName.endsWith('.js')) return Icons.javascript;
    if (lowerName.endsWith('.json')) return Icons.data_object;
    if (lowerName.endsWith('.db') || lowerName.endsWith('.sqlite')) return Icons.storage;
    if (lowerName.endsWith('.md')) return Icons.insert_drive_file;
    if (lowerName.endsWith('.txt')) return Icons.text_snippet;

    if (lowerName.endsWith('.pdf')) return Icons.picture_as_pdf;
    if (lowerName.endsWith('.doc') || lowerName.endsWith('.docx')) return Icons.description;
    if (lowerName.endsWith('.xls') || lowerName.endsWith('.xlsx') || lowerName.endsWith('.csv')) return Icons.table_chart;
    if (lowerName.endsWith('.png') || lowerName.endsWith('.jpg') || lowerName.endsWith('.jpeg') || lowerName.endsWith('.webp') || lowerName.endsWith('.bmp') || lowerName.endsWith('.gif')) {
      return Icons.image;
    }

    return Icons.insert_drive_file;
  }

  Color _fileIconColor(String name) {
    final lowerName = name.toLowerCase();
    if (lowerName.endsWith('.rpl')) return const Color(0xFF519ABA);
    if (lowerName.endsWith('.html')) return const Color(0xFFE44D26);
    if (lowerName.endsWith('.css')) return const Color(0xFF42A5F5);
    if (lowerName.endsWith('.js')) return const Color(0xFFDCDCAA);
    if (lowerName.endsWith('.json')) return const Color(0xFFDCDCAA);

    if (lowerName.endsWith('.pdf')) return const Color(0xFFE53935);
    if (lowerName.endsWith('.doc') || lowerName.endsWith('.docx')) return const Color(0xFF1E88E5);
    if (lowerName.endsWith('.xls') || lowerName.endsWith('.xlsx') || lowerName.endsWith('.csv')) return const Color(0xFF43A047);
    if (lowerName.endsWith('.png') || lowerName.endsWith('.jpg') || lowerName.endsWith('.jpeg') || lowerName.endsWith('.webp') || lowerName.endsWith('.bmp') || lowerName.endsWith('.gif')) {
      return const Color(0xFFAB47BC);
    }

    return const Color(0xFFCCCCCC);
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
