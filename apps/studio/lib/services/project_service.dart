import 'dart:convert';
import 'dart:io';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:archive/archive.dart';
import 'package:path_provider/path_provider.dart';
import '../models/project.dart';

/// Service untuk manajemen project: create, open, recent list.
class ProjectService {
  static const _recentProjectsKey = 'recent_projects';
  static const _maxRecent = 10;

  /// Load daftar recent projects dari shared_preferences.
  static Future<List<Project>> getRecentProjects() async {
    final prefs = await SharedPreferences.getInstance();
    final json = prefs.getStringList(_recentProjectsKey) ?? [];
    final projects = json
        .map((s) => Project.fromJson(jsonDecode(s) as Map<String, dynamic>))
        .toList();
    // Sort by lastOpened descending
    projects.sort((a, b) => b.lastOpened.compareTo(a.lastOpened));
    return projects;
  }

  /// Simpan project ke recent list.
  static Future<void> addToRecent(Project project) async {
    final prefs = await SharedPreferences.getInstance();
    final existing = await getRecentProjects();

    // Remove existing entry with same path
    existing.removeWhere((p) => p.path == project.path);

    // Insert at beginning
    existing.insert(0, project);

    // Keep max _maxRecent
    if (existing.length > _maxRecent) {
      existing.removeRange(_maxRecent, existing.length);
    }

    final json = existing.map((p) => jsonEncode(p.toJson())).toList();
    await prefs.setStringList(_recentProjectsKey, json);
  }

  /// Touch — update lastOpened time.
  static Future<void> touchProject(Project project) async {
    await addToRecent(project.copyWith(lastOpened: DateTime.now()));
  }

  /// Hapus project dari recent list.
  static Future<void> removeFromRecent(String path) async {
    final prefs = await SharedPreferences.getInstance();
    final existing = await getRecentProjects();
    existing.removeWhere((p) => p.path == path);
    final json = existing.map((p) => jsonEncode(p.toJson())).toList();
    await prefs.setStringList(_recentProjectsKey, json);
  }

  /// Export project folder ke file ZIP.
  static Future<String> exportToZip(String projectPath) async {
    final projectDir = Directory(projectPath);
    if (!projectDir.existsSync()) {
      throw Exception('Folder project tidak ditemukan.');
    }

    final projectName = projectPath.split(Platform.pathSeparator).last;
    final archive = Archive();

    // Scan all files recursively
    for (var entity in projectDir.listSync(recursive: true)) {
      if (entity is File) {
        final relativePath = entity.path.substring(projectPath.length + 1);
        final name = relativePath.split(Platform.pathSeparator).last;
        // Skip hidden files and build folders
        if (name.startsWith('.')) continue;
        if (relativePath.startsWith('build')) continue;

        try {
          final bytes = entity.readAsBytesSync();
          archive.addFile(ArchiveFile(
            relativePath.replaceAll(Platform.pathSeparator, '/'),
            bytes.length,
            bytes,
          ));
        } catch (_) {
          // Skip unreadable files
        }
      }
    }

    final docsDir = await getApplicationDocumentsDirectory();
    final zipFileName = '${projectName}_${DateTime.now().millisecondsSinceEpoch}.zip';
    final zipPath = '${docsDir.path}${Platform.pathSeparator}$zipFileName';

    final zipData = ZipEncoder().encode(archive);
    if (zipData != null) {
      File(zipPath).writeAsBytesSync(zipData);
    } else {
      throw Exception('Gagal membuat file ZIP.');
    }

    return zipPath;
  }

  /// Legacy createProject (backward compat)
  static Future<Project> createProject({
    required String name,
    required String parentPath,
    required ProjectTemplate template,
  }) async {
    final projectPath = '$parentPath${Platform.pathSeparator}$name';

    final dir = Directory(projectPath);
    if (await dir.exists()) {
      throw Exception('Folder "$projectPath" sudah ada.');
    }
    await dir.create(recursive: true);

    switch (template) {
      case ProjectTemplate.console:
        await File(
          '$projectPath/main.rpl',
        ).writeAsString(_consoleTemplate(name));
        break;
      default:
        await File(
          '$projectPath/main.rpl',
        ).writeAsString(_consoleTemplate(name));
        break;
    }

    final project = Project(
      name: name,
      path: projectPath,
      template: template,
      createdAt: DateTime.now(),
      lastOpened: DateTime.now(),
    );

    await addToRecent(project);
    return project;
  }

  /// Buat project dari ProjectTemplateDefinition baru
  static Future<Project> createProjectFromTemplate({
    required String name,
    required String parentPath,
    required ProjectTemplateDefinition templateDef,
    bool useTypeScript = false,
  }) async {
    final projectPath = '$parentPath${Platform.pathSeparator}$name';

    final dir = Directory(projectPath);
    if (await dir.exists()) {
      throw Exception('Folder "$name" sudah ada di lokasi tersebut.');
    }
    await dir.create(recursive: true);

    // Generate file berdasarkan template
    await _generateTemplateFiles(projectPath, templateDef, name, useTypeScript);

    final project = Project(
      name: name,
      path: projectPath,
      template: ProjectTemplate.console, // backward compat
      createdAt: DateTime.now(),
      lastOpened: DateTime.now(),
    );

    await addToRecent(project);
    return project;
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // Template file generators
  // ═══════════════════════════════════════════════════════════════════════════

  static Future<void> _generateTemplateFiles(
    String projectPath,
    ProjectTemplateDefinition templateDef,
    String projectName,
    bool useTypeScript,
  ) async {
    switch (templateDef.id) {
      // ── Rakoda ────────────────────────────────────────────────────
      case 'rakoda_empty':
        await File('$projectPath/main.rpl').writeAsString(
          'tampilkan "Halo Dunia dari $projectName!"\n',
        );
        break;

      // ── HTML ──────────────────────────────────────────────────────
      case 'html_empty':
        await File('$projectPath/index.html').writeAsString(_htmlTemplate(projectName));
        await File('$projectPath/style.css').writeAsString(_cssTemplate());
        break;

      // ── PHP ───────────────────────────────────────────────────────
      case 'php_empty':
        await File('$projectPath/index.php').writeAsString(_phpTemplate(projectName));
        break;
      case 'php_ci3':
        // CI3 tidak tersedia via Composer, buat struktur dasar
        await _generateCi3Template(projectPath, projectName);
        break;
      case 'php_wordpress':
      case 'php_ci4':
      case 'php_laravel12':
      case 'php_laravel10':
        // Template dengan CLI command — buat placeholder readme dulu
        await File('$projectPath/README.md').writeAsString(
          '# $projectName\n\n'
          'Template: ${templateDef.name}\n\n'
          'Jalankan perintah berikut di terminal untuk mengunduh framework:\n\n'
          '```\n${templateDef.cliCommand}\n```\n',
        );
        break;

      // ── Node.js ───────────────────────────────────────────────────
      case 'node_empty':
        await File('$projectPath/index.js').writeAsString(_nodeTemplate(projectName));
        break;
      case 'node_express':
        await _generateExpressTemplate(projectPath, projectName, useTypeScript);
        break;
      case 'node_react':
      case 'node_vue':
      case 'node_next':
      case 'node_nest':
        // Template dengan CLI command — buat placeholder readme
        await File('$projectPath/README.md').writeAsString(
          '# $projectName\n\n'
          'Template: ${templateDef.name}${useTypeScript ? ' (TypeScript)' : ''}\n\n'
          'Jalankan perintah berikut di terminal untuk mengunduh framework:\n\n'
          '```\n${templateDef.cliCommand}${useTypeScript ? ' --typescript' : ''}\n```\n',
        );
        break;

      // ── Python ────────────────────────────────────────────────────
      case 'python_empty':
        await File('$projectPath/main.py').writeAsString(_pythonTemplate(projectName));
        break;

      // ── Java ──────────────────────────────────────────────────────
      case 'java_empty':
        await File('$projectPath/Main.java').writeAsString(_javaTemplate(projectName));
        break;

      default:
        // Fallback: buat file Rakoda default
        await File('$projectPath/main.rpl').writeAsString(
          'tampilkan "Halo Dunia!"\n',
        );
    }
  }

  //#region Templates
  static String _consoleTemplate(String name) => '''tampilkan "Halo Dunia"''';

  static String _htmlTemplate(String projectName) => '''<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>$projectName</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>Selamat Datang di $projectName! 🎉</h1>
        <p>Edit file <code>index.html</code> untuk memulai.</p>
    </div>
</body>
</html>
''';

  static String _cssTemplate() => '''* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #1a1a2e;
    color: #eee;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 100vh;
}

.container {
    text-align: center;
    padding: 2rem;
}

h1 {
    font-size: 2rem;
    margin-bottom: 1rem;
    background: linear-gradient(135deg, #667eea, #764ba2);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
}

code {
    background: rgba(255, 255, 255, 0.1);
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.9em;
}
''';

  static String _phpTemplate(String projectName) => '''<?php
/**
 * $projectName
 * Dibuat dengan RPL Studio
 */

echo "Halo Dunia dari $projectName!\\n";
''';

  static String _nodeTemplate(String projectName) => '''// $projectName
// Dibuat dengan RPL Studio

console.log("Halo Dunia dari $projectName! 🚀");
''';

  static String _pythonTemplate(String projectName) => '''# $projectName
# Dibuat dengan RPL Studio

def main():
    print(f"Halo Dunia dari $projectName! 🐍")

if __name__ == "__main__":
    main()
''';

  static String _javaTemplate(String projectName) => '''// $projectName
// Dibuat dengan RPL Studio

public class Main {
    public static void main(String[] args) {
        System.out.println("Halo Dunia dari $projectName! ☕");
    }
}
''';

  static Future<void> _generateCi3Template(String projectPath, String projectName) async {
    // Buat struktur folder dasar CodeIgniter 3
    await Directory('$projectPath/application/controllers').create(recursive: true);
    await Directory('$projectPath/application/models').create(recursive: true);
    await Directory('$projectPath/application/views').create(recursive: true);
    await Directory('$projectPath/application/config').create(recursive: true);

    await File('$projectPath/index.php').writeAsString('''<?php
/**
 * $projectName - CodeIgniter 3
 * Dibuat dengan RPL Studio
 *
 * Unduh CodeIgniter 3 dari: https://codeigniter.com/download
 * Lalu ekstrak dan timpa folder ini.
 */

echo "Silakan unduh CodeIgniter 3 terlebih dahulu.\\n";
echo "Kunjungi: https://codeigniter.com/download\\n";
''');

    await File('$projectPath/README.md').writeAsString(
      '# $projectName\n\n'
      'Template: CodeIgniter 3\n\n'
      '## Cara Memulai\n\n'
      '1. Unduh CodeIgniter 3 dari https://codeigniter.com/download\n'
      '2. Ekstrak dan timpa folder project ini\n'
      '3. Jalankan `php -S localhost:8000` di terminal\n',
    );
  }

  static Future<void> _generateExpressTemplate(
    String projectPath, String projectName, bool useTypeScript,
  ) async {
    if (useTypeScript) {
      await File('$projectPath/package.json').writeAsString(jsonEncode({
        'name': projectName.toLowerCase().replaceAll(' ', '-'),
        'version': '1.0.0',
        'scripts': {
          'dev': 'npx tsx watch src/index.ts',
          'build': 'npx tsc',
          'start': 'node dist/index.js',
        },
        'dependencies': {
          'express': '^4.21.0',
        },
        'devDependencies': {
          '@types/express': '^4.17.21',
          '@types/node': '^22.0.0',
          'tsx': '^4.19.0',
          'typescript': '^5.6.0',
        },
      }));

      await Directory('$projectPath/src').create(recursive: true);
      await File('$projectPath/src/index.ts').writeAsString('''import express from "express";

const app = express();
const PORT = 3000;

app.get("/", (req, res) => {
  res.json({ message: "Halo Dunia dari $projectName! 🚀" });
});

app.listen(PORT, () => {
  console.log(\`Server berjalan di http://localhost:\${PORT}\`);
});
''');

      await File('$projectPath/tsconfig.json').writeAsString(jsonEncode({
        'compilerOptions': {
          'target': 'ES2022',
          'module': 'commonjs',
          'outDir': './dist',
          'rootDir': './src',
          'strict': true,
          'esModuleInterop': true,
        },
        'include': ['src/**/*'],
      }));
    } else {
      await File('$projectPath/package.json').writeAsString(jsonEncode({
        'name': projectName.toLowerCase().replaceAll(' ', '-'),
        'version': '1.0.0',
        'scripts': {
          'dev': 'node --watch index.js',
          'start': 'node index.js',
        },
        'dependencies': {
          'express': '^4.21.0',
        },
      }));

      await File('$projectPath/index.js').writeAsString('''const express = require("express");

const app = express();
const PORT = 3000;

app.get("/", (req, res) => {
  res.json({ message: "Halo Dunia dari $projectName! 🚀" });
});

app.listen(PORT, () => {
  console.log(\`Server berjalan di http://localhost:\${PORT}\`);
});
''');
    }

    await File('$projectPath/README.md').writeAsString(
      '# $projectName\n\n'
      'Template: Express${useTypeScript ? ' (TypeScript)' : ''}\n\n'
      '## Cara Memulai\n\n'
      '1. Jalankan `npm install` di terminal\n'
      '2. Jalankan `npm run dev` untuk memulai server\n'
      '3. Buka http://localhost:3000 di browser\n',
    );
  }
  //#endregion
}
