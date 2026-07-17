import 'package:flutter/material.dart';
import '../../models/database_connection.dart';
import 'package:uuid/uuid.dart';
import 'package:path/path.dart' as p;

class ConnectionDialog extends StatefulWidget {
  final String projectPath;
  final DatabaseConnection? connection;

  const ConnectionDialog({super.key, required this.projectPath, this.connection});

  @override
  State<ConnectionDialog> createState() => _ConnectionDialogState();
}

class _ConnectionDialogState extends State<ConnectionDialog> {
  final _formKey = GlobalKey<FormState>();

  late TextEditingController _nameController;
  late DatabaseEngine _engine;
  late TextEditingController _hostController;
  late TextEditingController _portController;
  late TextEditingController _userController;
  late TextEditingController _passController;
  late TextEditingController _dbController;
  late TextEditingController _sqlitePathController;

  @override
  void initState() {
    super.initState();
    final c = widget.connection;
    _nameController = TextEditingController(text: c?.name ?? '');
    _engine = c?.engine ?? DatabaseEngine.sqlite;
    _hostController = TextEditingController(text: c?.host ?? '');
    _portController = TextEditingController(text: c?.port?.toString() ?? '');
    _userController = TextEditingController(text: c?.username ?? '');
    _passController = TextEditingController(text: c?.password ?? '');
    _dbController = TextEditingController(text: c?.database ?? '');
    final pathStr = c?.sqlitePath ?? '';
    _sqlitePathController = TextEditingController(text: pathStr.isNotEmpty ? p.basename(pathStr) : '');
  }

  @override
  void dispose() {
    _nameController.dispose();
    _hostController.dispose();
    _portController.dispose();
    _userController.dispose();
    _passController.dispose();
    _dbController.dispose();
    _sqlitePathController.dispose();
    super.dispose();
  }

  void _onEngineChanged(DatabaseEngine? value) {
    if (value == null) return;
    setState(() {
      _engine = value;
      if (value == DatabaseEngine.mysql && _portController.text.isEmpty) {
        _portController.text = '3306';
      } else if (value == DatabaseEngine.postgres &&
          _portController.text.isEmpty) {
        _portController.text = '5432';
      }
    });
  }

  void _save() {
    if (!_formKey.currentState!.validate()) return;

    final connection = DatabaseConnection(
      id: widget.connection?.id ?? const Uuid().v4(),
      name: _nameController.text,
      engine: _engine,
      host: _hostController.text.isNotEmpty ? _hostController.text : null,
      port: int.tryParse(_portController.text),
      username: _userController.text.isNotEmpty ? _userController.text : null,
      password: _passController.text.isNotEmpty ? _passController.text : null,
      database: _dbController.text.isNotEmpty ? _dbController.text : null,
      sqlitePath: p.join(
        widget.projectPath,
        _sqlitePathController.text.trim().isNotEmpty
            ? _sqlitePathController.text.trim()
            : 'database.db',
      ),
    );

    Navigator.of(context).pop(connection);
  }

  @override
  Widget build(BuildContext context) {
    final isSqlite = _engine == DatabaseEngine.sqlite;

    return AlertDialog(
      backgroundColor: const Color(0xFF252526),
      title: Text(
        widget.connection == null ? 'New Connection' : 'Edit Connection',
        style: const TextStyle(color: Colors.white),
      ),
      content: SizedBox(
        width: 400,
        child: Form(
          key: _formKey,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _buildField('Connection Name', _nameController, required: true),
                const SizedBox(height: 16),
                const Text(
                  'Database Engine',
                  style: TextStyle(color: Colors.white70, fontSize: 12),
                ),
                const SizedBox(height: 8),
                DropdownButtonFormField<DatabaseEngine>(
                  value: _engine,
                  dropdownColor: const Color(0xFF333333),
                  style: const TextStyle(color: Colors.white),
                  decoration: const InputDecoration(
                    isDense: true,
                    filled: true,
                    fillColor: Color(0xFF1E1E1E),
                    border: OutlineInputBorder(borderSide: BorderSide.none),
                  ),
                  items: DatabaseEngine.values.map((e) {
                    return DropdownMenuItem(
                      value: e,
                      child: Text(e.name.toUpperCase()),
                    );
                  }).toList(),
                  onChanged: _onEngineChanged,
                ),
                const SizedBox(height: 16),

                if (isSqlite) ...[
                  _buildField(
                    'SQLite File Name (e.g. data.db)',
                    _sqlitePathController,
                    hintText: 'Leave empty for default (database.db)',
                  ),
                ] else ...[
                  Row(
                    children: [
                      Expanded(
                        flex: 3,
                        child: _buildField(
                          'Host',
                          _hostController,
                          required: true,
                        ),
                      ),
                      const SizedBox(width: 16),
                      Expanded(
                        flex: 1,
                        child: _buildField('Port', _portController),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  Row(
                    children: [
                      Expanded(child: _buildField('Username', _userController)),
                      const SizedBox(width: 16),
                      Expanded(
                        child: _buildField(
                          'Password',
                          _passController,
                          obscureText: true,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  _buildField('Default Database', _dbController),
                ],
              ],
            ),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel', style: TextStyle(color: Colors.white54)),
        ),
        ElevatedButton(
          style: ElevatedButton.styleFrom(
            backgroundColor: const Color(0xFF007ACC),
          ),
          onPressed: _save,
          child: const Text('Save', style: TextStyle(color: Colors.white)),
        ),
      ],
    );
  }

  Widget _buildField(
    String label,
    TextEditingController controller, {
    bool required = false,
    bool obscureText = false,
    String? hintText,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(color: Colors.white70, fontSize: 12),
        ),
        const SizedBox(height: 8),
        TextFormField(
          controller: controller,
          obscureText: obscureText,
          style: const TextStyle(color: Colors.white, fontSize: 14),
          decoration: InputDecoration(
            isDense: true,
            filled: true,
            fillColor: const Color(0xFF1E1E1E),
            border: const OutlineInputBorder(borderSide: BorderSide.none),
            hintText: hintText,
            hintStyle: const TextStyle(color: Colors.white24, fontSize: 13),
          ),
          validator: required
              ? (v) => v == null || v.isEmpty ? 'Required' : null
              : null,
        ),
      ],
    );
  }
}
