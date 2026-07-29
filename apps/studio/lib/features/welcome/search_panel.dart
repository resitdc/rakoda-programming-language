import 'dart:io';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import '../../shared/file_icon_helper.dart';

class FileMatch {
  final String filePath;
  final String fileName;
  final List<LineMatch> matches;
  bool isExpanded;

  FileMatch({
    required this.filePath,
    required this.fileName,
    required this.matches,
    this.isExpanded = true,
  });
}

class LineMatch {
  final int lineNumber;
  final String lineText;
  final int indexInLine;
  final int matchLength;

  LineMatch({
    required this.lineNumber,
    required this.lineText,
    required this.indexInLine,
    required this.matchLength,
  });
}

class SearchPanel extends StatefulWidget {
  final String rootPath;
  final void Function(String filePath, int lineNumber)? onMatchTap;

  const SearchPanel({
    super.key,
    required this.rootPath,
    this.onMatchTap,
  });

  @override
  State<SearchPanel> createState() => _SearchPanelState();
}

class _SearchPanelState extends State<SearchPanel> {
  final _searchController = TextEditingController();
  final _replaceController = TextEditingController();
  bool _isReplaceExpanded = false;
  bool _searching = false;
  List<FileMatch> _results = [];

  @override
  void initState() {
    super.initState();
    _searchController.addListener(_onSearchChanged);
  }

  @override
  void dispose() {
    _searchController.removeListener(_onSearchChanged);
    _searchController.dispose();
    _replaceController.dispose();
    super.dispose();
  }

  void _onSearchChanged() {
    _performSearch(_searchController.text);
  }

  void _performSearch(String query) {
    if (query.isEmpty) {
      setState(() {
        _results = [];
        _searching = false;
      });
      return;
    }

    setState(() => _searching = true);

    final results = <FileMatch>[];
    try {
      final dir = Directory(widget.rootPath);
      if (dir.existsSync()) {
        _searchInDirectory(dir, query, results);
      }
    } catch (_) {}

    setState(() {
      _results = results;
      _searching = false;
    });
  }

  void _searchInDirectory(Directory dir, String query, List<FileMatch> results) {
    for (var entity in dir.listSync()) {
      final name = entity.path.split(Platform.pathSeparator).last;
      if (name.startsWith('.')) continue;

      if (entity is Directory) {
        if (name == 'build' || name == 'ios' || name == 'android' || name == 'macos' || name == 'node_modules') {
          continue;
        }
        _searchInDirectory(entity, query, results);
      } else if (entity is File) {
        final ext = name.split('.').last.toLowerCase();
        final isBinary = ['png', 'jpg', 'jpeg', 'gif', 'pdf', 'zip', 'tar', 'gz', 'exe', 'dll', 'so', 'dylib', 'db', 'sqlite'].contains(ext);
        if (isBinary) continue;

        try {
          final lines = entity.readAsLinesSync();
          final lineMatches = <LineMatch>[];
          for (int i = 0; i < lines.length; i++) {
            final line = lines[i];
            int index = line.toLowerCase().indexOf(query.toLowerCase());
            if (index >= 0) {
              lineMatches.add(LineMatch(
                lineNumber: i + 1,
                lineText: line,
                indexInLine: index,
                matchLength: query.length,
              ));
            }
          }
          if (lineMatches.isNotEmpty) {
            results.add(FileMatch(
              filePath: entity.path,
              fileName: name,
              matches: lineMatches,
            ));
          }
        } catch (_) {}
      }
    }
  }

  void _performReplaceAll() {
    final query = _searchController.text;
    final replacement = _replaceController.text;
    if (query.isEmpty) return;

    int replaceCount = 0;
    int fileCount = 0;

    for (var fileMatch in _results) {
      try {
        final file = File(fileMatch.filePath);
        if (file.existsSync()) {
          final content = file.readAsStringSync();
          final matches = query.toLowerCase().allMatches(content.toLowerCase()).length;
          if (matches > 0) {
            final newContent = content.replaceAll(RegExp(query, caseSensitive: false), replacement);
            file.writeAsStringSync(newContent);
            replaceCount += matches;
            fileCount++;
          }
        }
      } catch (_) {}
    }

    _performSearch(query);

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('$replaceCount kecocokan diganti di $fileCount file'),
        backgroundColor: const Color(0xFF333333),
      ),
    );
  }

  int get _totalMatchCount {
    int total = 0;
    for (var f in _results) {
      total += f.matches.length;
    }
    return total;
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF252526),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 8),
            child: Text(
              'SEARCH',
              style: TextStyle(
                color: Colors.white.withOpacity(0.6),
                fontSize: 11,
                fontWeight: FontWeight.w600,
                letterSpacing: 0.5,
              ),
            ),
          ),
          // Search Input
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(
              children: [
                GestureDetector(
                  onTap: () => setState(() => _isReplaceExpanded = !_isReplaceExpanded),
                  child: Icon(
                    _isReplaceExpanded ? Icons.expand_more : Icons.chevron_right,
                    size: 14,
                    color: Colors.white38,
                  ),
                ),
                const SizedBox(width: 4),
                Expanded(child: _buildSearchField(_searchController, 'Search')),
              ],
            ),
          ),
          // Replace Input
          if (_isReplaceExpanded)
            Padding(
              padding: const EdgeInsets.only(left: 30, right: 12, top: 6),
              child: Row(
                children: [
                  Expanded(child: _buildSearchField(_replaceController, 'Replace')),
                  const SizedBox(width: 4),
                  _SmallButton(
                    icon: Icons.find_replace,
                    tooltip: 'Replace All',
                    onPressed: _performReplaceAll,
                  ),
                ],
              ),
            ),
          // Results summary
          if (_searchController.text.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
              child: Text(
                _searching
                    ? 'Mencari...'
                    : '${_totalMatchCount} hasil di ${_results.length} file',
                style: TextStyle(
                  color: Colors.white.withOpacity(0.35),
                  fontSize: 11,
                ),
              ),
            ),
          const SizedBox(height: 4),
          // Results List
          Expanded(
            child: _searching
                ? const Center(child: CircularProgressIndicator(color: Color(0xFF2568E7), strokeWidth: 2))
                : _results.isEmpty
                    ? Center(
                        child: Text(
                          _searchController.text.isEmpty ? '' : 'Tidak ditemukan',
                          style: TextStyle(color: Colors.white.withOpacity(0.25), fontSize: 12),
                        ),
                      )
                    : ListView.builder(
                        itemCount: _results.length,
                        itemBuilder: (context, i) => _buildFileMatchGroup(_results[i]),
                      ),
          ),
        ],
      ),
    );
  }

  Widget _buildSearchField(TextEditingController controller, String hint) {
    return SizedBox(
      height: 26,
      child: TextField(
        controller: controller,
        style: const TextStyle(fontSize: 12, color: Colors.white),
        decoration: InputDecoration(
          hintText: hint,
          hintStyle: TextStyle(color: Colors.white.withOpacity(0.3)),
          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 0),
          filled: true,
          fillColor: const Color(0xFF3C3C3C),
          border: const OutlineInputBorder(borderSide: BorderSide(color: Colors.transparent)),
          enabledBorder: const OutlineInputBorder(borderSide: BorderSide(color: Colors.transparent)),
          focusedBorder: const OutlineInputBorder(borderSide: BorderSide(color: Color(0xFF2568E7))),
        ),
      ),
    );
  }

  Widget _buildFileMatchGroup(FileMatch fileMatch) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => setState(() => fileMatch.isExpanded = !fileMatch.isExpanded),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
            child: Row(
              children: [
                Icon(
                  fileMatch.isExpanded ? Icons.expand_more : Icons.chevron_right,
                  size: 12,
                  color: Colors.white38,
                ),
                const SizedBox(width: 4),
                  FileIconHelper.getFileIcon(fileMatch.fileName, size: 14),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    fileMatch.fileName,
                    style: const TextStyle(
                      color: Colors.white70,
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                  decoration: BoxDecoration(
                    color: const Color(0xFF3C3C3C),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(
                    '${fileMatch.matches.length}',
                    style: const TextStyle(color: Colors.white54, fontSize: 10),
                  ),
                ),
              ],
            ),
          ),
        ),
        if (fileMatch.isExpanded)
          ...fileMatch.matches.map((lineMatch) {
            return InkWell(
              onTap: () => widget.onMatchTap?.call(fileMatch.filePath, lineMatch.lineNumber),
              hoverColor: const Color(0xFF2A2D2E),
              child: Padding(
                padding: const EdgeInsets.only(left: 32, right: 12, top: 2, bottom: 2),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    SizedBox(
                      width: 32,
                      child: Text(
                        '${lineMatch.lineNumber}',
                        style: const TextStyle(color: Colors.white24, fontSize: 11, fontFamily: 'monospace'),
                        textAlign: TextAlign.right,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: _buildHighlightedLine(
                        lineMatch.lineText.trim(),
                        _searchController.text,
                      ),
                    ),
                  ],
                ),
              ),
            );
          }),
      ],
    );
  }

  Widget _buildHighlightedLine(String text, String query) {
    if (query.isEmpty) {
      return Text(
        text,
        style: const TextStyle(color: Colors.white54, fontSize: 11, fontFamily: 'monospace'),
        overflow: TextOverflow.ellipsis,
      );
    }

    final matches = query.toLowerCase().allMatches(text.toLowerCase());
    if (matches.isEmpty) {
      return Text(
        text,
        style: const TextStyle(color: Colors.white54, fontSize: 11, fontFamily: 'monospace'),
        overflow: TextOverflow.ellipsis,
      );
    }

    List<TextSpan> spans = [];
    int lastMatchEnd = 0;
    for (var match in matches) {
      if (match.start > lastMatchEnd) {
        spans.add(TextSpan(
          text: text.substring(lastMatchEnd, match.start),
          style: const TextStyle(color: Colors.white54),
        ));
      }
      spans.add(TextSpan(
        text: text.substring(match.start, match.end),
        style: const TextStyle(
          color: Colors.white,
          backgroundColor: Color(0xFF623A18),
          fontWeight: FontWeight.bold,
        ),
      ));
      lastMatchEnd = match.end;
    }

    if (lastMatchEnd < text.length) {
      spans.add(TextSpan(
        text: text.substring(lastMatchEnd),
        style: const TextStyle(color: Colors.white54),
      ));
    }

    return RichText(
      overflow: TextOverflow.ellipsis,
      text: TextSpan(
        style: const TextStyle(fontSize: 11, fontFamily: 'monospace'),
        children: spans,
      ),
    );
  }


}

class _SmallButton extends StatelessWidget {
  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;

  const _SmallButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon, size: 14, color: Colors.white54),
      onPressed: onPressed,
      padding: EdgeInsets.zero,
      constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
      tooltip: tooltip,
    );
  }
}
