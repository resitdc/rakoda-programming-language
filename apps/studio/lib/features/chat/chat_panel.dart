import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'chat_service.dart';
import 'package:hugeicons/hugeicons.dart';

class ChatPanel extends StatefulWidget {
  const ChatPanel({Key? key}) : super(key: key);

  @override
  State<ChatPanel> createState() => _ChatPanelState();
}

class _ChatPanelState extends State<ChatPanel> {
  final ChatService _chatService = ChatService();
  final TextEditingController _codeController = TextEditingController();
  final TextEditingController _messageController = TextEditingController();
  final ScrollController _scrollController = ScrollController();
  
  ChatStatus _status = ChatStatus.disconnected;
  List<ChatMessage> _messages = [];
  bool _isLoading = false;

  @override
  void initState() {
    super.initState();
    _chatService.onStatusChanged.listen((status) {
      if (mounted) setState(() => _status = status);
    });
    _chatService.onMessage.listen((msg) {
      if (mounted) {
        setState(() => _messages.add(msg));
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
    _chatService.dispose();
    _codeController.dispose();
    _messageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _hostRoom() async {
    setState(() => _isLoading = true);
    try {
      await _chatService.hostRoom();
      _messages.clear();
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
      await _chatService.joinRoom(code);
      _messages.clear();
    } catch (e) {
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(e.toString())));
    } finally {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  void _sendMessage() {
    final text = _messageController.text.trim();
    if (text.isEmpty) return;
    _chatService.sendMessage(text);
    _messageController.clear();
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
            child: _status == ChatStatus.disconnected ? _buildLobby() : _buildRoom(),
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
            'LAN Chat',
            style: TextStyle(color: Colors.white, fontSize: 13, fontWeight: FontWeight.w600),
          ),
          const Spacer(),
          if (_status != ChatStatus.disconnected)
            IconButton(
              icon: const Icon(Icons.logout, color: Colors.white54, size: 16),
              tooltip: 'Keluar',
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(),
              onPressed: () => _chatService.disconnect(),
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
          SizedBox(
            width: double.infinity,
            child: ElevatedButton(
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.blueAccent,
                foregroundColor: Colors.white,
              ),
              onPressed: _isLoading ? null : _hostRoom,
              child: _isLoading 
                ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                : const Text("Buat Room"),
            ),
          ),
          const SizedBox(height: 24),
          const Row(
            children: [
              Expanded(child: Divider(color: Colors.white12)),
              Padding(
                padding: EdgeInsets.symmetric(horizontal: 8),
                child: Text("ATAU", style: TextStyle(color: Colors.white38, fontSize: 11)),
              ),
              Expanded(child: Divider(color: Colors.white12)),
            ],
          ),
          const SizedBox(height: 24),
          TextField(
            controller: _codeController,
            style: const TextStyle(color: Colors.white, fontSize: 13),
            decoration: InputDecoration(
              hintText: 'Kode Room...',
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
            child: OutlinedButton(
              style: OutlinedButton.styleFrom(
                foregroundColor: Colors.white,
                side: const BorderSide(color: Colors.white24),
              ),
              onPressed: _isLoading ? null : _joinRoom,
              child: const Text("Gabung"),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRoom() {
    return Column(
      children: [
        if (_status == ChatStatus.hosting && _chatService.roomCode != null)
          Container(
            padding: const EdgeInsets.all(12),
            margin: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              color: const Color(0xFF1E1E1E),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: Colors.white12),
            ),
            child: Column(
              children: [
                const Text("Room Code", style: TextStyle(color: Colors.white54, fontSize: 11)),
                const SizedBox(height: 4),
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      _chatService.roomCode!,
                      style: const TextStyle(color: Colors.greenAccent, fontSize: 18, fontWeight: FontWeight.bold, letterSpacing: 2),
                    ),
                    const SizedBox(width: 8),
                    InkWell(
                      onTap: () {
                        Clipboard.setData(ClipboardData(text: _chatService.roomCode!));
                        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("Kode disalin!")));
                      },
                      child: const Icon(Icons.copy, size: 16, color: Colors.white70),
                    )
                  ],
                ),
                const SizedBox(height: 8),
                const Text("Menunggu teman bergabung...", style: TextStyle(color: Colors.white38, fontSize: 11)),
              ],
            ),
          ),
        
        Expanded(
          child: ListView.builder(
            controller: _scrollController,
            padding: const EdgeInsets.all(8),
            itemCount: _messages.length,
            itemBuilder: (context, index) {
              final msg = _messages[index];
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 6),
                child: Row(
                  mainAxisAlignment: msg.isMine ? MainAxisAlignment.end : MainAxisAlignment.start,
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    if (!msg.isMine) ...[
                      CircleAvatar(
                        radius: 14,
                        backgroundColor: const Color(0xFF1E1E1E),
                        child: Text(msg.avatar, style: const TextStyle(fontSize: 16)),
                      ),
                      const SizedBox(width: 8),
                    ],
                    Flexible(
                      child: Column(
                        crossAxisAlignment: msg.isMine ? CrossAxisAlignment.end : CrossAxisAlignment.start,
                        children: [
                          if (!msg.isMine)
                            Padding(
                              padding: const EdgeInsets.only(left: 4, bottom: 4),
                              child: Text(msg.name, style: const TextStyle(color: Colors.white54, fontSize: 11)),
                            ),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                            decoration: BoxDecoration(
                              color: msg.isMine ? Colors.blueAccent.withAlpha(50) : const Color(0xFF333333),
                              borderRadius: BorderRadius.circular(12).copyWith(
                                bottomLeft: Radius.circular(msg.isMine ? 12 : 2),
                                bottomRight: Radius.circular(msg.isMine ? 2 : 12),
                              ),
                              border: Border.all(color: msg.isMine ? Colors.blueAccent.withAlpha(125) : Colors.white12),
                            ),
                            child: Text(
                              msg.text,
                              style: const TextStyle(color: Colors.white, fontSize: 13),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              );
            },
          ),
        ),
        
        if (_status == ChatStatus.connected)
          Container(
            padding: const EdgeInsets.all(8),
            decoration: const BoxDecoration(
              color: Color(0xFF1E1E1E),
              border: Border(top: BorderSide(color: Color(0xFF333333))),
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _messageController,
                    style: const TextStyle(color: Colors.white, fontSize: 13),
                    onSubmitted: (_) => _sendMessage(),
                    decoration: InputDecoration(
                      hintText: 'Ketik pesan...',
                      hintStyle: const TextStyle(color: Colors.white38),
                      filled: true,
                      fillColor: const Color(0xFF2D2D2D),
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(4),
                        borderSide: BorderSide.none,
                      ),
                      contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                IconButton(
                  icon: const Icon(Icons.send, color: Colors.blueAccent),
                  onPressed: _sendMessage,
                ),
              ],
            ),
          ),
      ],
    );
  }
}
