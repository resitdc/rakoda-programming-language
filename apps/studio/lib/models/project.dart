/// Model untuk project RPL Studio.
library;

enum ProjectTemplate { console, website, restApi, desktop, library, cli }

extension ProjectTemplateExtension on ProjectTemplate {
  String get displayName {
    switch (this) {
      case ProjectTemplate.console:
        return 'Console';
      case ProjectTemplate.website:
        return 'Website';
      case ProjectTemplate.restApi:
        return 'REST API';
      case ProjectTemplate.desktop:
        return 'Desktop';
      case ProjectTemplate.library:
        return 'Library';
      case ProjectTemplate.cli:
        return 'CLI';
    }
  }

  String get description {
    switch (this) {
      case ProjectTemplate.console:
        return 'Program sederhana yang berjalan di terminal';
      case ProjectTemplate.website:
        return 'Website dengan HTML, CSS, dan backend RPL';
      case ProjectTemplate.restApi:
        return 'Backend REST API dengan routing dan database';
      case ProjectTemplate.desktop:
        return 'Aplikasi desktop native';
      case ProjectTemplate.library:
        return 'Pustaka/modul yang bisa dipakai ulang';
      case ProjectTemplate.cli:
        return 'Command-line tool';
    }
  }

  String get icon {
    switch (this) {
      case ProjectTemplate.console:
        return 'terminal';
      case ProjectTemplate.website:
        return 'web';
      case ProjectTemplate.restApi:
        return 'api';
      case ProjectTemplate.desktop:
        return 'desktop_windows';
      case ProjectTemplate.library:
        return 'library_books';
      case ProjectTemplate.cli:
        return 'code';
    }
  }
}

class Project {
  final String name;
  final String path;
  final ProjectTemplate template;
  final DateTime createdAt;
  final DateTime lastOpened;

  Project({
    required this.name,
    required this.path,
    required this.template,
    required this.createdAt,
    required this.lastOpened,
  });

  Map<String, dynamic> toJson() => {
    'name': name,
    'path': path,
    'template': template.name,
    'createdAt': createdAt.toIso8601String(),
    'lastOpened': lastOpened.toIso8601String(),
  };

  factory Project.fromJson(Map<String, dynamic> json) => Project(
    name: json['name'] as String,
    path: json['path'] as String,
    template: ProjectTemplate.values.firstWhere(
      (t) => t.name == json['template'],
      orElse: () => ProjectTemplate.console,
    ),
    createdAt: DateTime.parse(json['createdAt'] as String),
    lastOpened: DateTime.parse(json['lastOpened'] as String),
  );

  Project copyWith({DateTime? lastOpened}) => Project(
    name: name,
    path: path,
    template: template,
    createdAt: createdAt,
    lastOpened: lastOpened ?? this.lastOpened,
  );
}


// ─── Template Definition System ─────────────────────────────────────────────

/// Kategori filter untuk template
enum TemplateCategory {
  semua,
  rakoda,
  html,
  php,
  node,
  python,
  java,
}

extension TemplateCategoryExt on TemplateCategory {
  String get label {
    switch (this) {
      case TemplateCategory.semua: return 'Semua';
      case TemplateCategory.rakoda: return 'Rakoda';
      case TemplateCategory.html: return 'HTML';
      case TemplateCategory.php: return 'PHP';
      case TemplateCategory.node: return 'Node.js';
      case TemplateCategory.python: return 'Python';
      case TemplateCategory.java: return 'Java';
    }
  }

  /// Runtime key yang sesuai di RuntimeManager (null = selalu tersedia)
  String? get runtimeKey {
    switch (this) {
      case TemplateCategory.semua: return null;
      case TemplateCategory.rakoda: return null;
      case TemplateCategory.html: return null;
      case TemplateCategory.php: return 'php';
      case TemplateCategory.node: return 'node';
      case TemplateCategory.python: return 'python';
      case TemplateCategory.java: return 'java';
    }
  }
}

/// Definisi satu template project
class ProjectTemplateDefinition {
  final String id;
  final String name;
  final String description;
  final TemplateCategory category;
  final String tag;            // Ditampilkan sebagai badge (e.g. "RPL", "PHP", "JS/TS")
  final int tagColor;          // Warna ARGB badge
  final String iconAsset;      // Path file ekstensi untuk FileIconHelper (e.g. ".rpl", ".php")
  final bool hasJsTsToggle;    // Apakah template ini punya opsi JavaScript/TypeScript
  final String? cliCommand;    // Perintah CLI untuk generate project (null = generate file manual)
  final String? downloadUrl;   // URL zip untuk diunduh otomatis
  final String? extractSubfolder; // Folder akar di dalam zip yang isinya akan diekstrak ke project

  const ProjectTemplateDefinition({
    required this.id,
    required this.name,
    required this.description,
    required this.category,
    required this.tag,
    required this.tagColor,
    required this.iconAsset,
    this.hasJsTsToggle = false,
    this.cliCommand,
    this.downloadUrl,
    this.extractSubfolder,
  });
}

/// Semua template yang tersedia
const List<ProjectTemplateDefinition> allProjectTemplates = [
  // ── Selalu Tersedia ──────────────────────────────────────────────
  ProjectTemplateDefinition(
    id: 'rakoda_empty',
    name: 'Kosong (Rakoda)',
    description: 'Project kosong untuk bahasa Rakoda',
    category: TemplateCategory.rakoda,
    tag: 'RPL',
    tagColor: 0xFF4EC9B0,
    iconAsset: '.rpl',
  ),
  ProjectTemplateDefinition(
    id: 'html_empty',
    name: 'Kosong (HTML)',
    description: 'Halaman web HTML dasar dengan CSS',
    category: TemplateCategory.html,
    tag: 'HTML',
    tagColor: 0xFFE44D26,
    iconAsset: '.html',
  ),

  // ── PHP (membutuhkan runtime PHP) ────────────────────────────────
  ProjectTemplateDefinition(
    id: 'php_empty',
    name: 'Kosong (PHP)',
    description: 'File PHP sederhana',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFF777BB4,
    iconAsset: '.php',
  ),
  ProjectTemplateDefinition(
    id: 'php_wordpress',
    name: 'WordPress',
    description: 'CMS WordPress terbaru',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFF777BB4,
    iconAsset: '.php',
    downloadUrl: 'https://wordpress.org/latest.zip',
    extractSubfolder: 'wordpress',
  ),
  ProjectTemplateDefinition(
    id: 'php_ci4',
    name: 'CodeIgniter 4',
    description: 'Framework PHP modern (MVC)',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFFDD4814,
    iconAsset: '.php',
    downloadUrl: 'https://github.com/codeigniter4/framework/archive/refs/tags/v4.4.7.zip',
    extractSubfolder: 'framework-4.4.7',
  ),
  ProjectTemplateDefinition(
    id: 'php_ci3',
    name: 'CodeIgniter 3',
    description: 'Framework PHP klasik',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFFDD4814,
    iconAsset: '.php',
    downloadUrl: 'https://github.com/bcit-ci/CodeIgniter/archive/refs/tags/3.1.13.zip',
    extractSubfolder: 'CodeIgniter-3.1.13',
  ),
  ProjectTemplateDefinition(
    id: 'php_laravel12',
    name: 'Laravel 12',
    description: 'Framework PHP full-stack terbaru',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFFFF2D20,
    iconAsset: '.php',
    downloadUrl: 'https://github.com/laravel/laravel/archive/refs/heads/11.x.zip', // 12.x not out yet, fallback to 11.x codebase
    extractSubfolder: 'laravel-11.x',
  ),
  ProjectTemplateDefinition(
    id: 'php_laravel10',
    name: 'Laravel 10',
    description: 'Framework PHP LTS',
    category: TemplateCategory.php,
    tag: 'PHP',
    tagColor: 0xFFFF2D20,
    iconAsset: '.php',
    downloadUrl: 'https://github.com/laravel/laravel/archive/refs/heads/10.x.zip',
    extractSubfolder: 'laravel-10.x',
  ),

  // ── Node.js (membutuhkan runtime Node) ───────────────────────────
  ProjectTemplateDefinition(
    id: 'node_empty',
    name: 'Kosong (Node.js)',
    description: 'File JavaScript sederhana',
    category: TemplateCategory.node,
    tag: 'JS',
    tagColor: 0xFFF7DF1E,
    iconAsset: '.js',
  ),
  ProjectTemplateDefinition(
    id: 'node_react',
    name: 'React',
    description: 'Library UI dari Meta',
    category: TemplateCategory.node,
    tag: 'JS/TS',
    tagColor: 0xFF61DAFB,
    iconAsset: '.js',
    hasJsTsToggle: true,
    cliCommand: 'npx create-react-app .',
  ),
  ProjectTemplateDefinition(
    id: 'node_vue',
    name: 'Vue 3',
    description: 'Framework progresif',
    category: TemplateCategory.node,
    tag: 'JS/TS',
    tagColor: 0xFF42B883,
    iconAsset: '.js',
    hasJsTsToggle: true,
    cliCommand: 'npx create-vue@latest .',
  ),
  ProjectTemplateDefinition(
    id: 'node_next',
    name: 'Next.js',
    description: 'React framework full-stack',
    category: TemplateCategory.node,
    tag: 'JS/TS',
    tagColor: 0xFFFFFFFF,
    iconAsset: '.js',
    hasJsTsToggle: true,
    cliCommand: 'npx create-next-app@latest .',
  ),
  ProjectTemplateDefinition(
    id: 'node_express',
    name: 'Express',
    description: 'Backend minimalis',
    category: TemplateCategory.node,
    tag: 'JS/TS',
    tagColor: 0xFF68A063,
    iconAsset: '.js',
    hasJsTsToggle: true,
  ),
  ProjectTemplateDefinition(
    id: 'node_nest',
    name: 'NestJS',
    description: 'Backend enterprise',
    category: TemplateCategory.node,
    tag: 'JS/TS',
    tagColor: 0xFFE0234E,
    iconAsset: '.js',
    hasJsTsToggle: true,
    cliCommand: 'npx @nestjs/cli new .',
  ),

  // ── Python (membutuhkan runtime Python) ──────────────────────────
  ProjectTemplateDefinition(
    id: 'python_empty',
    name: 'Kosong (Python)',
    description: 'File Python sederhana',
    category: TemplateCategory.python,
    tag: 'PY',
    tagColor: 0xFF3776AB,
    iconAsset: '.py',
  ),

  // ── Java (membutuhkan runtime Java) ──────────────────────────────
  ProjectTemplateDefinition(
    id: 'java_empty',
    name: 'Kosong (Java)',
    description: 'File Java sederhana',
    category: TemplateCategory.java,
    tag: 'JAVA',
    tagColor: 0xFFED8B00,
    iconAsset: '.java',
  ),
];
