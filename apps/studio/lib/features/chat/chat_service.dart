import 'dart:io';
import 'dart:convert';
import 'dart:async';
import 'dart:typed_data';

import '../identity/identity_service.dart';

enum ChatStatus { disconnected, hosting, connected }

class ChatMessage {
  final String uuid;
  final String name;
  final String avatar;
  final String text;
  final bool isMine;
  final DateTime timestamp;

  ChatMessage({
    required this.uuid,
    required this.name,
    required this.avatar,
    required this.text,
    required this.isMine,
  }) : timestamp = DateTime.now();

  Map<String, dynamic> toJson() => {
    'uuid': uuid,
    'name': name,
    'avatar': avatar,
    'text': text,
  };

  factory ChatMessage.fromJson(Map<String, dynamic> json, {required bool isMine}) {
    return ChatMessage(
      uuid: json['uuid'] ?? '',
      name: json['name'] ?? 'Unknown',
      avatar: json['avatar'] ?? '🐼',
      text: json['text'] ?? '',
      isMine: isMine,
    );
  }
}

class ChatService {
  ServerSocket? _serverSocket;
  Socket? _socket;
  
  ChatStatus status = ChatStatus.disconnected;
  String? roomCode;

  final _messageController = StreamController<ChatMessage>.broadcast();
  Stream<ChatMessage> get onMessage => _messageController.stream;

  final _statusController = StreamController<ChatStatus>.broadcast();
  Stream<ChatStatus> get onStatusChanged => _statusController.stream;

  void _updateStatus(ChatStatus s) {
    status = s;
    _statusController.add(s);
  }

  // --- Utility Room Code ---
  Future<String?> _getLocalIp() async {
    final interfaces = await NetworkInterface.list();
    for (var interface in interfaces) {
      for (var addr in interface.addresses) {
        if (addr.type == InternetAddressType.IPv4 && !addr.isLoopback) {
          return addr.address;
        }
      }
    }
    return null;
  }

  String _encodeRoomCode(String ip, int port) {
    final parts = ip.split('.');
    final b = BytesBuilder();
    for (var p in parts) {
      b.addByte(int.parse(p));
    }
    b.addByte(port >> 8);
    b.addByte(port & 0xFF);
    return base64Url.encode(b.toBytes()).replaceAll('=', '');
  }

  List<dynamic>? _decodeRoomCode(String code) {
    try {
      final normalized = code + ('=' * ((4 - code.length % 4) % 4));
      final decodedBytes = base64Url.decode(normalized);
      if (decodedBytes.length != 6) return null;
      final ip = '${decodedBytes[0]}.${decodedBytes[1]}.${decodedBytes[2]}.${decodedBytes[3]}';
      final port = (decodedBytes[4] << 8) | decodedBytes[5];
      return [ip, port];
    } catch (e) {
      return null;
    }
  }

  // --- Hosting ---
  Future<void> hostRoom() async {
    try {
      final ip = await _getLocalIp();
      if (ip == null) throw Exception("Tidak ada koneksi jaringan lokal.");

      _serverSocket = await ServerSocket.bind(InternetAddress.anyIPv4, 0);
      roomCode = _encodeRoomCode(ip, _serverSocket!.port);
      _updateStatus(ChatStatus.hosting);

      _serverSocket!.listen((client) {
        if (_socket != null) {
          client.close();
          return;
        }
        _socket = client;
        _listenToSocket();
        _updateStatus(ChatStatus.connected);
      });
    } catch (e) {
      disconnect();
      rethrow;
    }
  }

  // --- Joining ---
  Future<void> joinRoom(String code) async {
    try {
      final decoded = _decodeRoomCode(code.trim());
      if (decoded == null) throw Exception("Kode Room tidak valid.");
      
      final ip = decoded[0] as String;
      final port = decoded[1] as int;

      _socket = await Socket.connect(ip, port, timeout: const Duration(seconds: 5));
      roomCode = code.trim();
      _listenToSocket();
      _updateStatus(ChatStatus.connected);
    } catch (e) {
      disconnect();
      rethrow;
    }
  }

  void _listenToSocket() {
    _socket!.listen(
      (List<int> data) {
        final text = utf8.decode(data).trim();
        if (text.isNotEmpty) {
          try {
            // Kita pisahkan multiple JSON lines jika ada pesan terkirim secara bersamaan
            final lines = text.split('\n');
            for (var line in lines) {
              if (line.isEmpty) continue;
              final jsonMap = jsonDecode(line);
              _messageController.add(ChatMessage.fromJson(jsonMap, isMine: false));
            }
          } catch (e) {
            // Abaikan jika bukan JSON (untuk backward compatibility sementara)
          }
        }
      },
      onDone: disconnect,
      onError: (_) => disconnect(),
    );
  }

  void sendMessage(String text) {
    if (_socket == null || status != ChatStatus.connected) return;
    
    final msg = ChatMessage(
      uuid: IdentityService.uuid,
      name: IdentityService.name ?? 'Unknown',
      avatar: IdentityService.avatar ?? '🐼',
      text: text,
      isMine: true,
    );
    
    final jsonString = jsonEncode(msg.toJson()) + '\n';
    _socket!.write(jsonString);
    _messageController.add(msg);
  }

  void disconnect() {
    _socket?.close();
    _socket = null;
    _serverSocket?.close();
    _serverSocket = null;
    roomCode = null;
    _updateStatus(ChatStatus.disconnected);
  }

  void dispose() {
    disconnect();
    _messageController.close();
    _statusController.close();
  }
}
