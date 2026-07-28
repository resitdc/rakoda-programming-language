void main() {
  var b = BigInt.parse('c0a801051f90', radix: 16);
  var encoded = b.toRadixString(36).toUpperCase();
  print('Encoded: $encoded');
  var b2 = BigInt.parse(encoded, radix: 36);
  var hex = b2.toRadixString(16).padLeft(12, '0');
  print('Hex: $hex');
}
