import { Prism } from 'prism-react-renderer';

Prism.languages.rakoda = {
  'comment': {
    pattern: /\/\/.*/,
    greedy: true
  },
  'string': {
    pattern: /(^|[^\\])"(?:\\.|[^"\\\r\n])*"(?!\s*:)/,
    lookbehind: true,
    greedy: true
  },
  'keyword': /\b(buat|tampilkan|jika|maka|selain|itu|selama|lakukan|untuk|setiap|dalam|kembalikan|fungsi|kosong|dan|atau|tidak)\b/,
  'boolean': /\b(benar|salah)\b/,
  'number': /\b\d+(?:\.\d+)?\b/,
  'operator': /[=+\-*/<>!]+/,
  'punctuation': /[.,()[\]{}]/,
  'function': /\b[a-zA-Z_]\w*(?=\s*\()/
};
