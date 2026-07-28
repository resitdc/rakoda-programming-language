import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'classroom_service.dart';
import 'package:hugeicons/hugeicons.dart';
import 'dart:io';
import 'dart:convert';
import 'package:file_picker/file_picker.dart';

class ClassroomPanel extends StatefulWidget {
  final String projectPath;
  const ClassroomPanel({Key? key, required this.projectPath}) : super(key: key);

  @override
  State<ClassroomPanel> createState() => _ClassroomPanelState();
}

class _ClassroomPanelState extends State<ClassroomPanel> with SingleTickerProviderStateMixin {
  final ClassroomService _classroomService = ClassroomService();
  final TextEditingController _codeController = TextEditingController();
  final TextEditingController _customCodeController = TextEditingController();
  final TextEditingController _messageController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  late final FocusNode _focusNode;
  late TabController _tabController;
  
  late ClassroomStatus _status = _classroomService.status;
  bool _isLoading = false;
  String? _viewingStudentUuid;
  bool _isJoinMode = true;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    
    _classroomService.initDatabase(widget.projectPath).then((_) {
      if (mounted) setState(() {});
    });
    
    if (_classroomService.roomCode != null && _status == ClassroomStatus.disconnected) {
      _codeController.text = _classroomService.roomCode!;
    }
    
    if (_classroomService.messages.isNotEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (_scrollController.hasClients) {
          _scrollController.jumpTo(_scrollController.position.maxScrollExtent);
        }
      });
    }

    _focusNode = FocusNode(
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent && event.logicalKey == LogicalKeyboardKey.enter) {
          if (!HardwareKeyboard.instance.isShiftPressed) {
            _sendMessage();
            return KeyEventResult.handled;
          }
        }
        return KeyEventResult.ignored;
      },
    );
    _classroomService.onStatusChanged.listen((status) {
      if (mounted) {
        final oldStatus = _status;
        setState(() => _status = status);
        if (oldStatus == ClassroomStatus.disconnected && 
            (status == ClassroomStatus.connected || status == ClassroomStatus.hosting)) {
          _tabController.animateTo(1);
        }
      }
    });
    _classroomService.onMessage.listen((msg) {
      if (mounted && msg.eventType == 'chat') {
        setState(() {});
        Future.delayed(const Duration(milliseconds: 100), () {
          if (_scrollController.hasClients) {
            _scrollController.animateTo(
              _scrollController.position.maxScrollExtent,
              duration: const Duration(milliseconds: 300),
              curve: Curves.easeOut,
            );
          }
        });
      }
    });
  }

  @override
  void dispose() {
    _focusNode.dispose();
    _codeController.dispose();
    _customCodeController.dispose();
    _messageController.dispose();
    _scrollController.dispose();
    _tabController.dispose();
    super.dispose();
  }

  void _hostRoom() async {
    setState(() => _isLoading = true);
    try {
      await _classroomService.hostRoom(customCode: _customCodeController.text);
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(e.toString())));
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  void _joinRoom() async {
    final code = _codeController.text.trim();
    if (code.isEmpty) return;
    
    setState(() => _isLoading = true);
    try {
      await _classroomService.joinRoom(code);
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(e.toString())));
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  void _sendMessage() {
    final text = _messageController.text.trim();
    if (text.isEmpty) {
      _messageController.clear();
      _focusNode.requestFocus();
      return;
    }
    
    _classroomService.sendMessage(text);
    _messageController.clear();
    _focusNode.requestFocus();
    
    Future.delayed(const Duration(milliseconds: 100), () {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 300),
          curve: Curves.easeOut,
        );
      }
    });
  }

  Future<void> _sendFile() async {
    final result = await FilePicker.pickFiles();
    if (result == null || result.files.isEmpty) return;

    final file = File(result.files.first.path!);
    final fileSize = await file.length();
    
    if (fileSize > 20 * 1024 * 1024) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("Ukuran file maksimal 20 MB")));
      return;
    }

    final bytes = await file.readAsBytes();
    final base64String = base64Encode(bytes);
    final fileName = result.files.first.name;

    _classroomService.sendMessage("📎 Lampiran File", fileName: fileName, fileBase64: base64String);
  }

  Future<void> _saveFile(ClassroomEvent msg) async {
    if (msg.fileName == null || msg.fileBase64 == null) return;
    
    final result = await FilePicker.saveFile(
      dialogTitle: 'Simpan Lampiran',
      fileName: msg.fileName,
    );
    
    if (result != null) {
      final bytes = base64Decode(msg.fileBase64!);
      final file = File(result);
      await file.writeAsBytes(bytes);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("File berhasil disimpan!")));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 300,
      color: const Color(0xFF252526),
      child: Column(
        children: [
          _buildHeader(),
          Expanded(
            child: _status == ClassroomStatus.disconnected ? _buildLobby() : _buildRoom(),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Row(
        children: [
          const HugeIcon(icon: HugeIcons.strokeRoundedChatting01, color: Colors.white70, size: 18),
          const SizedBox(width: 8),
          const Text(
            'Room',
            style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.w600),
          ),
          const Spacer(),
          if (_status != ClassroomStatus.disconnected)
            IconButton(
              icon: const Icon(Icons.logout, color: Colors.white54, size: 16),
              tooltip: 'Keluar',
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
              onPressed: () {
                _classroomService.disconnect();
                if (mounted) setState(() => _status = ClassroomStatus.disconnected);
              },
            ),
        ],
      ),
    );
  }

  Widget _buildLobby() {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.wifi, size: 48, color: Colors.white24),
          const SizedBox(height: 16),
          const Text(
            "Kolaborasi Jaringan Lokal",
            style: TextStyle(color: Colors.white70, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 8),
          const Text(
            "Chat dengan rekan satu WiFi tanpa internet.",
            textAlign: TextAlign.center,
            style: TextStyle(color: Colors.white54, fontSize: 12),
          ),
          const SizedBox(height: 32),
          
          if (_isJoinMode) ...[
            // JOIN MODE
            TextField(
              controller: _codeController,
              style: const TextStyle(color: Colors.white, fontSize: 13),
              decoration: InputDecoration(
                hintText: 'Masukkan Kode Room...',
                hintStyle: const TextStyle(color: Colors.white38),
                filled: true,
                fillColor: const Color(0xFF1E1E1E),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(4),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
              ),
            ),
            const SizedBox(height: 8),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.blueAccent,
                  foregroundColor: Colors.white,
                ),
                onPressed: _isLoading ? null : _joinRoom,
                child: _isLoading 
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                  : const Text("Gabung ke Room"),
              ),
            ),
          ] else ...[
            // HOST MODE
            TextField(
              controller: _customCodeController,
              style: const TextStyle(color: Colors.white, fontSize: 13),
              decoration: InputDecoration(
                hintText: 'Nama Room (Opsional)',
                hintStyle: const TextStyle(color: Colors.white38),
                filled: true,
                fillColor: const Color(0xFF1E1E1E),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(4),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
              ),
            ),
            const SizedBox(height: 8),
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                style: ElevatedButton.styleFrom(
                  backgroundColor: Colors.green,
                  foregroundColor: Colors.white,
                ),
                onPressed: _isLoading ? null : _hostRoom,
                child: _isLoading 
                  ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                  : const Text("Buat Room Baru"),
              ),
            ),
          ],

          const SizedBox(height: 24),
          const Divider(color: Colors.white12),
          const SizedBox(height: 8),
          
          TextButton(
            onPressed: () {
              setState(() {
                _isJoinMode = !_isJoinMode;
              });
            },
            child: Text(
              _isJoinMode ? "Saya seorang Guru, buat room" : "Saya seorang Siswa, gabung room",
              style: const TextStyle(color: Colors.white54, fontSize: 12),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRoom() {
    return Column(
      children: [
        // Room info bar: role + room code (compact)
        _buildRoomInfoBar(),
        
        // Teacher controls
        if (_classroomService.isHost) _buildTeacherControls(),
        
        // Tab bar
        _buildTabBar(),
        
        // Tab content
        Expanded(
          child: TabBarView(
            controller: _tabController,
            children: [
              _buildStudentsTab(),
              _buildChatTab(),
            ],
          ),
        ),
        
        // Message input (only on chat tab)
        if (_status == ClassroomStatus.connected || _status == ClassroomStatus.hosting)
          _buildMessageInput(),
      ],
    );
  }

  Widget _buildRoomInfoBar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Row(
        children: [
          if (_classroomService.isHost) ...[
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.amber.withAlpha(30),
                borderRadius: BorderRadius.circular(4),
              ),
              child: const Text(
                'HOST',
                style: TextStyle(
                  color: Colors.amber,
                  fontSize: 11,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            const SizedBox(width: 8),
          ],
          if (_classroomService.roomCode != null) ...[
            const Icon(Icons.tag, color: Colors.white24, size: 14),
            const SizedBox(width: 4),
            Expanded(
              child: Text(
                _classroomService.roomCode!,
                style: const TextStyle(color: Colors.greenAccent, fontSize: 12, fontWeight: FontWeight.w600, letterSpacing: 1),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            InkWell(
              onTap: () {
                Clipboard.setData(ClipboardData(text: _classroomService.roomCode!));
                ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("Kode disalin!")));
              },
              child: const Padding(
                padding: EdgeInsets.all(4),
                child: Icon(Icons.copy, size: 14, color: Colors.white54),
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTeacherControls() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      decoration: const BoxDecoration(
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Row(
        children: [
          const Icon(Icons.screen_share_outlined, color: Colors.white38, size: 14),
          const SizedBox(width: 8),
          const Text("Live Code", style: TextStyle(color: Colors.white54, fontSize: 12)),
          const Spacer(),
          SizedBox(
            height: 28,
            child: Transform.scale(
              scale: 0.75,
              child: Switch(
                value: _classroomService.isLiveCodeSharingEnabled,
                activeColor: Colors.blueAccent,
                onChanged: (val) {
                  setState(() {
                    _classroomService.setLiveCodeSharing(val);
                  });
                },
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTabBar() {
    return Container(
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: TabBar(
        controller: _tabController,
        indicatorColor: Colors.blueAccent,
        indicatorSize: TabBarIndicatorSize.tab,
        indicatorWeight: 2,
        labelColor: Colors.white,
        unselectedLabelColor: Colors.white38,
        labelStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
        unselectedLabelStyle: const TextStyle(fontSize: 12),
        dividerColor: Colors.transparent,
        tabs: [
          Tab(
            height: 32,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.people_outline, size: 14),
                const SizedBox(width: 6),
                Text('Siswa (${_classroomService.connectedStudents.length})'),
              ],
            ),
          ),
          Tab(
            height: 32,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.chat_bubble_outline, size: 14),
                const SizedBox(width: 6),
                Text('Chat (${_classroomService.messages.length})'),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildStudentsTab() {
    final students = _classroomService.connectedStudents;
    
    if (students.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.people_outline, size: 40, color: Colors.white12),
            const SizedBox(height: 12),
            Text(
              _classroomService.isHost 
                ? 'Belum ada siswa yang bergabung.\nBagikan kode room untuk mengundang.' 
                : 'Belum ada siswa lain di room ini.',
              textAlign: TextAlign.center,
              style: const TextStyle(color: Colors.white38, fontSize: 12),
            ),
          ],
        ),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.symmetric(vertical: 4),
      itemCount: students.length,
      separatorBuilder: (_, __) => const Divider(height: 1, color: Color(0xFF333333)),
      itemBuilder: (context, index) {
        final student = students[index];
        return _buildStudentTile(student);
      },
    );
  }

  Widget _buildStudentTile(ConnectedStudent student) {
    final isViewing = _viewingStudentUuid == student.uuid;
    return Material(
      color: isViewing ? Colors.blueAccent.withOpacity(0.15) : Colors.transparent,
      child: InkWell(
        onTap: _classroomService.isHost ? () {
          if (_viewingStudentUuid != null) {
            _classroomService.requestStudentCode(_viewingStudentUuid!, false);
          }
          setState(() {
            if (_viewingStudentUuid == student.uuid) {
              _viewingStudentUuid = null; // Unsubscribe / stop viewing
            } else {
              _viewingStudentUuid = student.uuid;
              _classroomService.requestStudentCode(student.uuid, true);
            }
          });
        } : null,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: [
              CircleAvatar(
                radius: 16,
                backgroundColor: const Color(0xFF2D2D2D),
                backgroundImage: NetworkImage('https://api.dicebear.com/9.x/adventurer/png?seed=${Uri.encodeComponent(student.avatar)}'),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      student.name,
                      style: const TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.w500),
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 2),
                    Row(
                      children: [
                        Container(
                          width: 6,
                          height: 6,
                          decoration: const BoxDecoration(
                            color: Colors.greenAccent,
                            shape: BoxShape.circle,
                          ),
                        ),
                        const SizedBox(width: 4),
                        const Text(
                          'Online',
                          style: TextStyle(color: Colors.white38, fontSize: 10),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              if (_classroomService.isHost)
                const Icon(Icons.chevron_right, color: Colors.white24, size: 16),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildChatTab() {
    if (_classroomService.messages.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.chat_bubble_outline, size: 40, color: Colors.white12),
            SizedBox(height: 12),
            Text(
              'Belum ada pesan.\nKirim pesan pertama!',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.white38, fontSize: 12),
            ),
          ],
        ),
      );
    }

    return ListView.builder(
      controller: _scrollController,
      padding: const EdgeInsets.all(12),
      itemCount: _classroomService.messages.length,
      itemBuilder: (context, index) {
        final msg = _classroomService.messages[index];
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: Row(
            mainAxisAlignment: msg.isMine ? MainAxisAlignment.end : MainAxisAlignment.start,
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              if (!msg.isMine) ...[
                CircleAvatar(
                  radius: 12,
                  backgroundColor: const Color(0xFF1E1E1E),
                  backgroundImage: NetworkImage('https://api.dicebear.com/9.x/adventurer/png?seed=${Uri.encodeComponent(msg.avatar)}'),
                ),
                const SizedBox(width: 6),
              ],
              Flexible(
                child: Column(
                  crossAxisAlignment: msg.isMine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
                  children: [
                    if (!msg.isMine)
                      Padding(
                        padding: const EdgeInsets.only(left: 4, bottom: 2),
                        child: Text(msg.name, style: const TextStyle(color: Colors.white54, fontSize: 10)),
                      ),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
                      decoration: BoxDecoration(
                        color: msg.isMine ? Colors.blueAccent.withAlpha(50) : const Color(0xFF333333),
                        borderRadius: BorderRadius.circular(10).copyWith(
                          bottomLeft: Radius.circular(msg.isMine ? 10 : 2),
                          bottomRight: Radius.circular(msg.isMine ? 2 : 10),
                        ),
                        border: Border.all(color: msg.isMine ? Colors.blueAccent.withAlpha(125) : Colors.white12),
                      ),
                      child: Column(
                        crossAxisAlignment: msg.isMine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
                        children: [
                          if (msg.fileName != null) ...[
                            Container(
                              padding: const EdgeInsets.all(6),
                              margin: const EdgeInsets.only(bottom: 6),
                              decoration: BoxDecoration(
                                color: Colors.black26,
                                borderRadius: BorderRadius.circular(6),
                              ),
                              child: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  const Icon(Icons.insert_drive_file, color: Colors.white70, size: 16),
                                  const SizedBox(width: 6),
                                  Flexible(
                                    child: Text(
                                      msg.fileName!,
                                      style: const TextStyle(color: Colors.white, fontSize: 11),
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ),
                                  const SizedBox(width: 6),
                                  InkWell(
                                    onTap: () => _saveFile(msg),
                                    child: const Icon(Icons.download, color: Colors.blueAccent, size: 16),
                                  ),
                                ],
                              ),
                            ),
                          ],
                          Text(
                            msg.text,
                            style: const TextStyle(color: Colors.white, fontSize: 12),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildMessageInput() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: const BoxDecoration(
        color: Color(0xFF1E1E1E),
        border: Border(top: BorderSide(color: Color(0xFF2A2A2A))),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          // Attach button
          Material(
            color: Colors.transparent,
            child: InkWell(
              onTap: _sendFile,
              borderRadius: BorderRadius.circular(16),
              child: const Padding(
                padding: EdgeInsets.all(6),
                child: Icon(Icons.attach_file_rounded, color: Colors.white38, size: 18),
              ),
            ),
          ),
          const SizedBox(width: 4),
          // Text field
          Expanded(
            child: TextField(
              controller: _messageController,
              focusNode: _focusNode,
              style: const TextStyle(color: Colors.white, fontSize: 13, height: 1.35),
              maxLines: 5,
              minLines: 1,
              cursorColor: Colors.white54,
              keyboardType: TextInputType.multiline,
              textInputAction: TextInputAction.newline,
              decoration: InputDecoration(
                hintText: 'Ketik pesan...',
                hintStyle: const TextStyle(color: Colors.white24, fontSize: 13),
                filled: true,
                fillColor: const Color(0xFF2D2D2D),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(20),
                  borderSide: BorderSide.none,
                ),
                enabledBorder: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(20),
                  borderSide: BorderSide.none,
                ),
                focusedBorder: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(20),
                  borderSide: BorderSide.none,
                ),
                contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
                isDense: true,
              ),
            ),
          ),
          const SizedBox(width: 4),
          // Send button
          Material(
            color: Colors.transparent,
            child: InkWell(
              onTap: _sendMessage,
              borderRadius: BorderRadius.circular(14),
              child: Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: const Color(0xFF00A884),
                  borderRadius: BorderRadius.circular(14),
                ),
                child: const Icon(Icons.send_rounded, color: Colors.white, size: 13),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
