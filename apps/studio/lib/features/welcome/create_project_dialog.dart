import 'dart:io';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path_provider/path_provider.dart';
import '../../models/project.dart';
import '../../services/project_service.dart';
import '../../shared/file_icon_helper.dart';
import '../editor/code_executor_service.dart';
import '../runtime/runtime_manager_screen.dart';

class CreateProjectDialog extends StatefulWidget {
  const CreateProjectDialog({super.key});

  @override
  State<CreateProjectDialog> createState() => _CreateProjectDialogState();
}

class _CreateProjectDialogState extends State<CreateProjectDialog> {
  // Step management
  int _currentStep = 0; // 0 = pilih template, 1 = konfigurasi project

  // Step 1 state
  TemplateCategory _selectedCategory = TemplateCategory.semua;
  ProjectTemplateDefinition? _selectedTemplate;
  final Set<String> _installedRuntimes = {};
  bool _loadingRuntimes = true;
  String _searchQuery = '';
  final _searchController = TextEditingController();

  // Step 2 state
  final _nameController = TextEditingController();
  String _parentPath = '';
  bool _useTypeScript = false;
  bool _creating = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _initDefaultPath();
    _detectInstalledRuntimes();
  }

  Future<void> _initDefaultPath() async {
    final appDir = await getApplicationDocumentsDirectory();
    final rplDir = '${appDir.path}${Platform.pathSeparator}RPLProjects';
    final dir = Directory(rplDir);
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    if (mounted) setState(() => _parentPath = rplDir);
  }

  Future<void> _detectInstalledRuntimes() async {
    final runtimes = <String>{};
    for (final lang in ['node', 'php', 'python', 'java']) {
      final paths = await CodeExecutorService.getInstalledRuntimePaths(lang);
      if (paths.isNotEmpty) {
        runtimes.add(lang);
      }
    }
    if (mounted) {
      setState(() {
        _installedRuntimes.addAll(runtimes);
        _loadingRuntimes = false;
      });
    }
  }

  bool _isCategoryAvailable(TemplateCategory cat) {
    final runtimeKey = cat.runtimeKey;
    if (runtimeKey == null) return true; // Rakoda & HTML selalu tersedia
    return _installedRuntimes.contains(runtimeKey);
  }

  List<ProjectTemplateDefinition> get _filteredTemplates {
    var templates = allProjectTemplates.where((t) {
      // Filter by category
      if (_selectedCategory != TemplateCategory.semua && t.category != _selectedCategory) {
        return false;
      }
      // Filter by search
      if (_searchQuery.isNotEmpty) {
        final q = _searchQuery.toLowerCase();
        return t.name.toLowerCase().contains(q) ||
            t.description.toLowerCase().contains(q) ||
            t.tag.toLowerCase().contains(q);
      }
      // Hide templates whose runtime is not installed (unless showing "Semua")
      if (_selectedCategory == TemplateCategory.semua) {
        return _isCategoryAvailable(t.category);
      }
      return true;
    }).toList();
    return templates;
  }

  Future<void> _pickFolder() async {
    final result = await FilePicker.getDirectoryPath();
    if (result != null) {
      setState(() => _parentPath = result);
    }
  }

  Future<void> _create() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      setState(() => _error = 'Nama project tidak boleh kosong.');
      return;
    }
    if (_parentPath.isEmpty) {
      setState(() => _error = 'Pilih folder penyimpanan terlebih dahulu.');
      return;
    }
    if (_selectedTemplate == null) {
      setState(() => _error = 'Pilih template terlebih dahulu.');
      return;
    }

    setState(() {
      _creating = true;
      _error = null;
    });

    try {
      final project = await ProjectService.createProjectFromTemplate(
        name: name,
        parentPath: _parentPath,
        templateDef: _selectedTemplate!,
        useTypeScript: _useTypeScript,
      );
      if (mounted) {
        Navigator.pop(context, project);
      }
    } catch (e) {
      setState(() {
        _creating = false;
        _error = e.toString();
      });
    }
  }

  void _goToStep2(ProjectTemplateDefinition template) {
    setState(() {
      _selectedTemplate = template;
      _currentStep = 1;
      _useTypeScript = false;
      _error = null;
    });
  }

  void _goBackToStep1() {
    setState(() {
      _currentStep = 0;
      _error = null;
    });
  }

  @override
  void dispose() {
    _nameController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final screenHeight = MediaQuery.of(context).size.height;
    final isSmallScreen = screenHeight < 700;

    return Dialog(
      backgroundColor: const Color(0xFF1E1E1E),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Container(
        constraints: BoxConstraints(
          maxWidth: 560,
          maxHeight: isSmallScreen ? screenHeight * 0.85 : 620,
        ),
        child: ClipRRect(
          borderRadius: BorderRadius.circular(16),
          child: AnimatedSwitcher(
            duration: const Duration(milliseconds: 250),
            transitionBuilder: (child, animation) {
              final slideIn = Tween<Offset>(
                begin: _currentStep == 1 ? const Offset(1, 0) : const Offset(-1, 0),
                end: Offset.zero,
              ).animate(CurvedAnimation(parent: animation, curve: Curves.easeOutCubic));
              return SlideTransition(position: slideIn, child: child);
            },
            child: _currentStep == 0
                ? _buildStep1(key: const ValueKey('step1'))
                : _buildStep2(key: const ValueKey('step2')),
          ),
        ),
      ),
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // STEP 1: Template Selection
  // ═══════════════════════════════════════════════════════════════════════════

  Widget _buildStep1({Key? key}) {
    return Column(
      key: key,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header ──
        _buildHeader('Buat Project Baru', Icons.create_new_folder_outlined),

        // ── Search bar ──
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
          child: Container(
            height: 40,
            decoration: BoxDecoration(
              color: const Color(0xFF252526),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xFF3C3C3C)),
            ),
            child: TextField(
              controller: _searchController,
              style: const TextStyle(color: Colors.white, fontSize: 13),
              decoration: InputDecoration(
                hintText: 'Cari template...',
                hintStyle: TextStyle(color: Colors.white.withOpacity(0.25), fontSize: 13),
                prefixIcon: Icon(Icons.search, color: Colors.white.withOpacity(0.3), size: 18),
                border: InputBorder.none,
                contentPadding: const EdgeInsets.symmetric(vertical: 10),
                suffixIcon: _searchQuery.isNotEmpty
                    ? IconButton(
                        icon: const Icon(Icons.close, size: 16, color: Colors.white38),
                        onPressed: () {
                          _searchController.clear();
                          setState(() => _searchQuery = '');
                        },
                      )
                    : null,
              ),
              onChanged: (v) => setState(() => _searchQuery = v),
            ),
          ),
        ),

        // ── Category filter chips ──
        SizedBox(
          height: 36,
          child: ListView(
            scrollDirection: Axis.horizontal,
            padding: const EdgeInsets.symmetric(horizontal: 16),
            children: TemplateCategory.values
                .where((cat) => _isCategoryAvailable(cat))
                .map((cat) {
              final isSelected = _selectedCategory == cat;

              return Padding(
                padding: const EdgeInsets.only(right: 8),
                child: _FilterChip(
                  label: cat.label,
                  isSelected: isSelected,
                  onTap: () => setState(() => _selectedCategory = cat),
                ),
              );
            }).toList(),
          ),
        ),

        const SizedBox(height: 12),

        // ── Template grid ──
        Expanded(
          child: _loadingRuntimes
              ? const Center(
                  child: CircularProgressIndicator(
                    color: Color(0xFF2568E7),
                    strokeWidth: 2.5,
                  ),
                )
              : _filteredTemplates.isEmpty
                  ? Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(Icons.search_off, color: Colors.white.withOpacity(0.15), size: 48),
                          const SizedBox(height: 12),
                          Text(
                            _searchQuery.isNotEmpty
                                ? 'Tidak ada template\nyang cocok'
                                : 'Tidak ada template\nuntuk kategori ini',
                            textAlign: TextAlign.center,
                            style: TextStyle(color: Colors.white.withOpacity(0.3), fontSize: 13),
                          ),
                        ],
                      ),
                    )
                  : GridView.builder(
                      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: 2,
                        mainAxisSpacing: 10,
                        crossAxisSpacing: 10,
                        childAspectRatio: 1.35,
                      ),
                      itemCount: _filteredTemplates.length,
                      itemBuilder: (ctx, i) {
                        final t = _filteredTemplates[i];
                        return _TemplateCard(
                          template: t,
                          onTap: () => _goToStep2(t),
                        );
                      },
                    ),
        ),
      ],
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // STEP 2: Project Configuration
  // ═══════════════════════════════════════════════════════════════════════════

  Widget _buildStep2({Key? key}) {
    final t = _selectedTemplate!;

    return Column(
      key: key,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // ── Header with back button ──
        Container(
          padding: const EdgeInsets.fromLTRB(16, 20, 16, 16),
          decoration: const BoxDecoration(
            border: Border(bottom: BorderSide(color: Color(0xFF2D2D30))),
          ),
          child: Row(
            children: [
              InkWell(
                onTap: _goBackToStep1,
                borderRadius: BorderRadius.circular(8),
                child: Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    color: Colors.white.withOpacity(0.06),
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Icon(Icons.arrow_back, color: Colors.white54, size: 18),
                ),
              ),
              const SizedBox(width: 12),
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: Color(t.tagColor).withOpacity(0.12),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Center(
                  child: FileIconHelper.getFileIcon('dummy${t.iconAsset}', size: 22),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      t.name,
                      style: const TextStyle(
                        color: Colors.white,
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    Text(
                      t.description,
                      style: TextStyle(color: Colors.white.withOpacity(0.4), fontSize: 11),
                    ),
                  ],
                ),
              ),
              // Tag badge
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                decoration: BoxDecoration(
                  color: Color(t.tagColor).withOpacity(0.15),
                  borderRadius: BorderRadius.circular(6),
                ),
                child: Text(
                  t.tag,
                  style: TextStyle(
                    color: Color(t.tagColor),
                    fontSize: 10,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ],
          ),
        ),

        // ── Form ──
        Expanded(
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Nama Project
                const Text(
                  'Nama Project',
                  style: TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 8),
                Container(
                  decoration: BoxDecoration(
                    color: const Color(0xFF252526),
                    borderRadius: BorderRadius.circular(10),
                    border: Border.all(color: const Color(0xFF3C3C3C)),
                  ),
                  child: TextField(
                    controller: _nameController,
                    autofocus: true,
                    style: const TextStyle(color: Colors.white, fontSize: 14),
                    decoration: InputDecoration(
                      hintText: 'contoh: aplikasi_toko',
                      hintStyle: TextStyle(color: Colors.white.withOpacity(0.2), fontSize: 13),
                      prefixIcon: Icon(Icons.edit_outlined, color: Colors.white.withOpacity(0.3), size: 18),
                      border: InputBorder.none,
                      contentPadding: const EdgeInsets.symmetric(vertical: 14, horizontal: 12),
                    ),
                  ),
                ),

                const SizedBox(height: 20),

                // Lokasi
                const Text(
                  'Lokasi',
                  style: TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                  decoration: BoxDecoration(
                    color: const Color(0xFF252526),
                    borderRadius: BorderRadius.circular(10),
                    border: Border.all(color: const Color(0xFF3C3C3C)),
                  ),
                  child: Row(
                    children: [
                      Icon(Icons.folder_outlined, size: 16, color: Colors.white.withOpacity(0.3)),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Text(
                          _parentPath.isEmpty ? 'Memilih folder...' : _parentPath,
                          style: TextStyle(color: Colors.white.withOpacity(0.45), fontSize: 12),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      GestureDetector(
                        onTap: _pickFolder,
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                          decoration: BoxDecoration(
                            color: const Color(0xFF2568E7).withOpacity(0.12),
                            borderRadius: BorderRadius.circular(6),
                          ),
                          child: const Text(
                            'Browse',
                            style: TextStyle(color: Color(0xFF2568E7), fontSize: 11, fontWeight: FontWeight.w600),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),

                // JS/TS toggle (hanya untuk template tertentu)
                if (t.hasJsTsToggle) ...[
                  const SizedBox(height: 20),
                  const Text(
                    'Bahasa',
                    style: TextStyle(color: Colors.white70, fontSize: 12, fontWeight: FontWeight.w600),
                  ),
                  const SizedBox(height: 8),
                  Container(
                    decoration: BoxDecoration(
                      color: const Color(0xFF252526),
                      borderRadius: BorderRadius.circular(10),
                      border: Border.all(color: const Color(0xFF3C3C3C)),
                    ),
                    child: Row(
                      children: [
                        Expanded(
                          child: GestureDetector(
                            onTap: () => setState(() => _useTypeScript = false),
                            child: Container(
                              padding: const EdgeInsets.symmetric(vertical: 12),
                              decoration: BoxDecoration(
                                color: !_useTypeScript
                                    ? const Color(0xFFF7DF1E).withOpacity(0.12)
                                    : Colors.transparent,
                                borderRadius: const BorderRadius.horizontal(left: Radius.circular(9)),
                              ),
                              child: Row(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  SizedBox(
                                    width: 16,
                                    height: 16,
                                    child: FileIconHelper.getFileIcon('dummy.js', size: 16),
                                  ),
                                  const SizedBox(width: 6),
                                  Text(
                                    'JavaScript',
                                    style: TextStyle(
                                      color: !_useTypeScript ? const Color(0xFFF7DF1E) : Colors.white38,
                                      fontSize: 13,
                                      fontWeight: !_useTypeScript ? FontWeight.w600 : FontWeight.w400,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                          ),
                        ),
                        Container(width: 1, height: 30, color: const Color(0xFF3C3C3C)),
                        Expanded(
                          child: GestureDetector(
                            onTap: () => setState(() => _useTypeScript = true),
                            child: Container(
                              padding: const EdgeInsets.symmetric(vertical: 12),
                              decoration: BoxDecoration(
                                color: _useTypeScript
                                    ? const Color(0xFF3178C6).withOpacity(0.12)
                                    : Colors.transparent,
                                borderRadius: const BorderRadius.horizontal(right: Radius.circular(9)),
                              ),
                              child: Row(
                                mainAxisAlignment: MainAxisAlignment.center,
                                children: [
                                  Icon(
                                    Icons.code,
                                    size: 16,
                                    color: _useTypeScript ? const Color(0xFF3178C6) : Colors.white38,
                                  ),
                                  const SizedBox(width: 6),
                                  Text(
                                    'TypeScript',
                                    style: TextStyle(
                                      color: _useTypeScript ? const Color(0xFF3178C6) : Colors.white38,
                                      fontSize: 13,
                                      fontWeight: _useTypeScript ? FontWeight.w600 : FontWeight.w400,
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
                ],

                // CLI command info
                if (t.cliCommand != null) ...[
                  const SizedBox(height: 16),
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: const Color(0xFF2568E7).withOpacity(0.06),
                      borderRadius: BorderRadius.circular(8),
                      border: Border.all(color: const Color(0xFF2568E7).withOpacity(0.15)),
                    ),
                    child: Row(
                      children: [
                        Icon(Icons.info_outline, size: 14, color: const Color(0xFF2568E7).withOpacity(0.7)),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            'Template ini akan diunduh dari internet saat pembuatan project.',
                            style: TextStyle(color: const Color(0xFF2568E7).withOpacity(0.7), fontSize: 11),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],

                // Error
                if (_error != null) ...[
                  const SizedBox(height: 16),
                  Container(
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: const Color(0xFF5A1D1D),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.error_outline, size: 14, color: Color(0xFFFF6B6B)),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            _error!,
                            style: const TextStyle(color: Color(0xFFFF6B6B), fontSize: 12),
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),

        // ── Bottom actions ──
        Container(
          padding: const EdgeInsets.fromLTRB(20, 12, 20, 20),
          decoration: const BoxDecoration(
            border: Border(top: BorderSide(color: Color(0xFF2D2D30))),
          ),
          child: Row(
            children: [
              Expanded(
                child: TextButton(
                  onPressed: _creating ? null : () => Navigator.pop(context),
                  style: TextButton.styleFrom(
                    foregroundColor: Colors.white54,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                      side: BorderSide(color: Colors.white.withOpacity(0.08)),
                    ),
                  ),
                  child: const Text('Batal'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                flex: 2,
                child: ElevatedButton(
                  onPressed: _creating ? null : _create,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF2568E7),
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                    elevation: 0,
                  ),
                  child: _creating
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                        )
                      : const Row(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.rocket_launch, size: 16),
                            SizedBox(width: 8),
                            Text('Buat Project', style: TextStyle(fontWeight: FontWeight.w600)),
                          ],
                        ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // Shared header widget
  // ═══════════════════════════════════════════════════════════════════════════

  Widget _buildHeader(String title, IconData icon) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 20, 16, 16),
      child: Row(
        children: [
          Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              color: const Color(0xFF2568E7).withOpacity(0.12),
              borderRadius: BorderRadius.circular(10),
            ),
            child: Icon(icon, color: const Color(0xFF2568E7), size: 20),
          ),
          const SizedBox(width: 12),
          Text(
            title,
            style: const TextStyle(
              fontSize: 18,
              fontWeight: FontWeight.bold,
              color: Colors.white,
            ),
          ),
        ],
      ),
    );
  }
}


// ═══════════════════════════════════════════════════════════════════════════
// Filter Chip widget
// ═══════════════════════════════════════════════════════════════════════════

class _FilterChip extends StatelessWidget {
  final String label;
  final bool isSelected;
  final VoidCallback onTap;

  const _FilterChip({
    required this.label,
    required this.isSelected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 200),
        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 7),
        decoration: BoxDecoration(
          color: isSelected
              ? const Color(0xFF2568E7).withOpacity(0.18)
              : Colors.white.withOpacity(0.06),
          borderRadius: BorderRadius.circular(20),
          border: Border.all(
            color: isSelected
                ? const Color(0xFF2568E7).withOpacity(0.5)
                : Colors.white.withOpacity(0.1),
          ),
        ),
        child: Text(
          label,
          style: TextStyle(
            color: isSelected ? const Color(0xFF2568E7) : Colors.white60,
            fontSize: 12,
            fontWeight: isSelected ? FontWeight.w600 : FontWeight.w400,
          ),
        ),
      ),
    );
  }
}


// ═══════════════════════════════════════════════════════════════════════════
// Template Card widget
// ═══════════════════════════════════════════════════════════════════════════

class _TemplateCard extends StatefulWidget {
  final ProjectTemplateDefinition template;
  final VoidCallback onTap;

  const _TemplateCard({required this.template, required this.onTap});

  @override
  State<_TemplateCard> createState() => _TemplateCardState();
}

class _TemplateCardState extends State<_TemplateCard> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final t = widget.template;

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          decoration: BoxDecoration(
            color: _hovering ? const Color(0xFF2A2A2E) : const Color(0xFF252526),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: _hovering
                  ? Color(t.tagColor).withOpacity(0.35)
                  : Colors.white.withOpacity(0.06),
              width: _hovering ? 1.5 : 1,
            ),
            boxShadow: _hovering
                ? [
                    BoxShadow(
                      color: Color(t.tagColor).withOpacity(0.08),
                      blurRadius: 12,
                      offset: const Offset(0, 4),
                    ),
                  ]
                : null,
          ),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    SizedBox(
                      width: 24,
                      height: 24,
                      child: FileIconHelper.getFileIcon('dummy${t.iconAsset}', size: 22),
                    ),
                    const Spacer(),
                    // Tag badge
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: Color(t.tagColor).withOpacity(0.12),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        t.tag,
                        style: TextStyle(
                          color: Color(t.tagColor),
                          fontSize: 9,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 0.3,
                        ),
                      ),
                    ),
                  ],
                ),
                const Spacer(),
                Text(
                  t.name,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                  ),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 3),
                Text(
                  t.description,
                  style: TextStyle(color: Colors.white.withOpacity(0.35), fontSize: 10),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
