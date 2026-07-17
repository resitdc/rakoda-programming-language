import 'dart:io';
import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:file_picker/file_picker.dart';
import '../../models/project.dart';
import '../../services/project_service.dart';
import 'create_project_dialog.dart';
import 'project_screen.dart';
import 'scan_barcode_screen.dart';

class WelcomeScreen extends StatefulWidget {
  const WelcomeScreen({super.key});

  @override
  State<WelcomeScreen> createState() => _WelcomeScreenState();
}

class _WelcomeScreenState extends State<WelcomeScreen> {
  List<Project> _recentProjects = [];
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadRecent();
  }

  Future<void> _loadRecent() async {
    final projects = await ProjectService.getRecentProjects();
    setState(() {
      _recentProjects = projects;
      _loading = false;
    });
  }

  Future<void> _createProject() async {
    final result = await showDialog<Project>(
      context: context,
      builder: (_) => const CreateProjectDialog(),
    );
    if (result != null && mounted) {
      Navigator.pushReplacement(
        context,
        MaterialPageRoute(builder: (_) => ProjectScreen(project: result)),
      );
    }
  }

  Future<void> _openFolder() async {
    final result = await FilePicker.getDirectoryPath();
    if (result != null && mounted) {
      final name = result.split(Platform.pathSeparator).last;
      final project = Project(
        name: name,
        path: result,
        template: ProjectTemplate.console,
        createdAt: DateTime.now(),
        lastOpened: DateTime.now(),
      );
      await ProjectService.touchProject(project);
      if (mounted) {
        Navigator.pushReplacement(
          context,
          MaterialPageRoute(builder: (_) => ProjectScreen(project: project)),
        );
      }
    }
  }

  Future<void> _openProject(Project project) async {
    await ProjectService.touchProject(project);
    if (mounted) {
      Navigator.pushReplacement(
        context,
        MaterialPageRoute(builder: (_) => ProjectScreen(project: project)),
      );
    }
  }

  Future<void> _removeFromRecent(Project project) async {
    await ProjectService.removeFromRecent(project.path);
    _loadRecent();
  }

  Future<void> _exportToZip(Project project) async {
    try {
      final zipPath = await ProjectService.exportToZip(project.path);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Berhasil diekspor ke: ${zipPath.split(Platform.pathSeparator).last}'),
            backgroundColor: const Color(0xFF007ACC),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Gagal mengekspor: $e'),
            backgroundColor: Colors.red[700],
          ),
        );
      }
    }
  }

  void _scanBarcode() {
    Navigator.push(
      context,
      MaterialPageRoute(builder: (_) => const ScanBarcodeScreen()),
    );
  }

  String _timeAgo(DateTime dt) {
    final diff = DateTime.now().difference(dt);
    if (diff.inMinutes < 1) return 'Baru saja';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m yang lalu';
    if (diff.inHours < 24) return '${diff.inHours}j yang lalu';
    if (diff.inDays < 7) return '${diff.inDays}h yang lalu';
    return '${diff.inDays ~/ 7}m yang lalu';
  }

  IconData _templateIcon(ProjectTemplate template) {
    switch (template) {
      case ProjectTemplate.website:
        return Icons.web;
      case ProjectTemplate.restApi:
        return Icons.api;
      case ProjectTemplate.desktop:
        return Icons.desktop_windows;
      case ProjectTemplate.library:
        return Icons.library_books;
      case ProjectTemplate.cli:
        return Icons.code;
      case ProjectTemplate.console:
        return Icons.terminal;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF1E1E1E),
      body: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 600),
            child: ListView(
              padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 40),
              children: [
                // === HEADER: Logo + Title ===
                _buildHeader(),
                const SizedBox(height: 48),

                // === ACTION CARDS ===
                _buildActionCards(),
                const SizedBox(height: 40),

                // === RECENT PROJECTS ===
                _buildRecentSection(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Column(
      children: [
        // Logo SVG
        SvgPicture.asset(
          'assets/rakoda-white.svg',
          width: 64,
          height: 64,
          colorFilter: const ColorFilter.mode(Colors.white, BlendMode.srcIn),
        ),
        const SizedBox(height: 16),
        const Text(
          'RPL STUDIO',
          style: TextStyle(
            fontSize: 28,
            fontWeight: FontWeight.bold,
            color: Colors.white,
            letterSpacing: 4,
          ),
        ),
        const SizedBox(height: 8),
        Text(
          'IDE Resmi Rakoda Programming Language',
          style: TextStyle(
            fontSize: 13,
            color: Colors.white.withOpacity(0.5),
          ),
        ),
      ],
    );
  }

  Widget _buildActionCards() {
    return GridView.count(
      crossAxisCount: 2,
      mainAxisSpacing: 12,
      crossAxisSpacing: 12,
      childAspectRatio: 1.6,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      children: [
        _ActionCard(
          icon: Icons.create_new_folder_outlined,
          label: 'Buat Project',
          subtitle: 'Buat project baru',
          color: const Color(0xFF007ACC),
          onTap: _createProject,
        ),
        _ActionCard(
          icon: Icons.folder_open_outlined,
          label: 'Buka Project',
          subtitle: 'Buka folder project',
          color: const Color(0xFF4EC9B0),
          onTap: _openFolder,
        ),
        _ActionCard(
          icon: Icons.qr_code_scanner_outlined,
          label: 'Scan Barcode',
          subtitle: 'Download sample project',
          color: const Color(0xFFDCDCAA),
          onTap: _scanBarcode,
        ),
        _ActionCard(
          icon: Icons.settings_outlined,
          label: 'Pengaturan',
          subtitle: 'Pengaturan aplikasi',
          color: const Color(0xFF9CDCFE),
          onTap: () {
            ScaffoldMessenger.of(context).showSnackBar(
              const SnackBar(
                content: Text('Pengaturan akan segera hadir!'),
                backgroundColor: Color(0xFF333333),
              ),
            );
          },
        ),
      ],
    );
  }

  Widget _buildRecentSection() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            const Icon(Icons.history, size: 16, color: Colors.white54),
            const SizedBox(width: 8),
            Text(
              'PROJECT TERAKHIR',
              style: TextStyle(
                fontSize: 11,
                fontWeight: FontWeight.bold,
                color: Colors.white.withOpacity(0.6),
                letterSpacing: 1.5,
              ),
            ),
          ],
        ),
        const SizedBox(height: 12),
        if (_loading)
          const Center(
            child: Padding(
              padding: EdgeInsets.all(32),
              child: CircularProgressIndicator(color: Color(0xFF007ACC)),
            ),
          )
        else if (_recentProjects.isEmpty)
          _buildEmptyState()
        else
          ..._recentProjects.map((project) => _buildRecentTile(project)),
      ],
    );
  }

  Widget _buildEmptyState() {
    return Container(
      padding: const EdgeInsets.all(32),
      decoration: BoxDecoration(
        color: const Color(0xFF252526),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xFF3C3C3C)),
      ),
      child: Column(
        children: [
          Icon(
            Icons.inbox_outlined,
            size: 48,
            color: Colors.white.withOpacity(0.15),
          ),
          const SizedBox(height: 12),
          Text(
            'Belum ada project terbaru',
            style: TextStyle(
              color: Colors.white.withOpacity(0.5),
              fontSize: 14,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            'Buat project baru atau buka folder untuk memulai',
            style: TextStyle(
              color: Colors.white.withOpacity(0.3),
              fontSize: 12,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRecentTile(Project project) {
    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      decoration: BoxDecoration(
        color: const Color(0xFF252526),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: const Color(0xFF3C3C3C)),
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: () => _openProject(project),
          borderRadius: BorderRadius.circular(6),
          hoverColor: const Color(0xFF2A2D2E),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
            child: Row(
              children: [
                // Template icon
                Container(
                  width: 36,
                  height: 36,
                  decoration: BoxDecoration(
                    color: const Color(0xFF333333),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Icon(
                    _templateIcon(project.template),
                    color: const Color(0xFF007ACC),
                    size: 18,
                  ),
                ),
                const SizedBox(width: 12),
                // Name + path + time
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        project.name,
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 13,
                          fontWeight: FontWeight.w500,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                      const SizedBox(height: 2),
                      Text(
                        '${project.path}  •  ${_timeAgo(project.lastOpened)}',
                        style: TextStyle(
                          color: Colors.white.withOpacity(0.35),
                          fontSize: 11,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ),
                // Export ZIP button
                IconButton(
                  icon: const Icon(Icons.archive_outlined, size: 16, color: Colors.white38),
                  tooltip: 'Export ke ZIP',
                  onPressed: () => _exportToZip(project),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
                ),
                // Remove button
                IconButton(
                  icon: const Icon(Icons.close, size: 14, color: Colors.white24),
                  tooltip: 'Hapus dari recent',
                  onPressed: () => _removeFromRecent(project),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

// === Action Card Widget ===
class _ActionCard extends StatefulWidget {
  final IconData icon;
  final String label;
  final String subtitle;
  final Color color;
  final VoidCallback onTap;

  const _ActionCard({
    required this.icon,
    required this.label,
    required this.subtitle,
    required this.color,
    required this.onTap,
  });

  @override
  State<_ActionCard> createState() => _ActionCardState();
}

class _ActionCardState extends State<_ActionCard> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
          decoration: BoxDecoration(
            color: const Color(0xFF252526),
            borderRadius: BorderRadius.circular(10),
            border: Border.all(
              color: _isHovered ? widget.color.withOpacity(0.6) : const Color(0xFF3C3C3C),
              width: _isHovered ? 1.5 : 1,
            ),
          ),
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(widget.icon, color: widget.color, size: 24),
              const SizedBox(height: 10),
              Text(
                widget.label,
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                widget.subtitle,
                style: TextStyle(
                  color: Colors.white.withOpacity(0.4),
                  fontSize: 11,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
