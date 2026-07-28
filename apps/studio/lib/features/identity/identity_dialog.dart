import 'package:flutter/material.dart';
import 'identity_service.dart';

class IdentityDialog extends StatefulWidget {
  final bool isCancellable;
  
  const IdentityDialog({Key? key, this.isCancellable = false}) : super(key: key);

  @override
  State<IdentityDialog> createState() => _IdentityDialogState();
}

class _IdentityDialogState extends State<IdentityDialog> {
  final TextEditingController _nameController = TextEditingController();
  @override
  void initState() {
    super.initState();
    if (IdentityService.name != null) {
      _nameController.text = IdentityService.name!;
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  void _save() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) return;

    await IdentityService.saveIdentity(name, name); // Gunakan nama sebagai seed
    if (mounted) {
      Navigator.of(context).pop(true);
    }
  }

  @override
  Widget build(BuildContext context) {
    return WillPopScope(
      onWillPop: () async => widget.isCancellable,
      child: Dialog(
        backgroundColor: const Color(0xFF252526),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        child: Container(
          width: 400,
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                widget.isCancellable ? "Ubah Profil" : "Selamat Datang di RPL Studio!",
                style: const TextStyle(color: Colors.white, fontSize: 18, fontWeight: FontWeight.bold),
              ),
              const SizedBox(height: 8),
              Text(
                widget.isCancellable 
                  ? "Pilih avatar dan nama baru Anda."
                  : "Sebelum memulai, mari buat profil Anda agar teman-teman mengenali Anda di Chat Room lokal.",
                style: const TextStyle(color: Colors.white70, fontSize: 13),
              ),
              const SizedBox(height: 24),
              const Text("Nama Tampilan:", style: TextStyle(color: Colors.white, fontSize: 13)),
              const SizedBox(height: 8),
              TextField(
                controller: _nameController,
                style: const TextStyle(color: Colors.white, fontSize: 14),
                decoration: InputDecoration(
                  filled: true,
                  fillColor: const Color(0xFF1E1E1E),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(6),
                    borderSide: BorderSide.none,
                  ),
                ),
                onSubmitted: (_) => _save(),
              ),
              const SizedBox(height: 32),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  if (widget.isCancellable) ...[
                    TextButton(
                      onPressed: () => Navigator.of(context).pop(false),
                      child: const Text("Batal", style: TextStyle(color: Colors.white54)),
                    ),
                    const SizedBox(width: 8),
                  ],
                  ElevatedButton(
                    style: ElevatedButton.styleFrom(
                      backgroundColor: Colors.blueAccent,
                      foregroundColor: Colors.white,
                      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
                    ),
                    onPressed: _save,
                    child: const Text("Simpan & Lanjutkan"),
                  ),
                ],
              )
            ],
          ),
        ),
      ),
    );
  }
}
