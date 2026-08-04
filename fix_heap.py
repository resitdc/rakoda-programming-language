import re

with open('crates/vm/src/heap.rs', 'r') as f:
    content = f.read()

# Replace all if let Value::DbPool(i) = val { c.push(*i); } with QueryState included
content = re.sub(
    r'(if let Value::DbPool\(i\) = val \{\s*c\.push\(\*i\);\s*\})',
    r'\1\n                        if let Value::QueryState(i) = val {\n                            c.push(*i);\n                        }',
    content
)

# Wait, the indentation in HeapData::Fungsi is different. It's better to just do this globally.
# Then add a match arm for HeapData::QueryState(q)
query_state_arm = """                HeapData::QueryState(q) => {
                    let mut c = Vec::new();
                    for (_, _, val) in &q.kondisi {
                        if let Value::Array(i) = val {
                            c.push(*i);
                        }
                        if let Value::Kamus(i) = val {
                            c.push(*i);
                        }
                        if let Value::String(i) = val {
                            c.push(*i);
                        }
                        if let Value::Fungsi(i, _) = val {
                            c.push(*i);
                        }
                        if let Value::FungsiBawaan(i) = val {
                            c.push(*i);
                        }
                        if let Value::Modul(i) = val {
                            c.push(*i);
                        }
                        if let Value::DbPool(i) = val {
                            c.push(*i);
                        }
                        if let Value::QueryState(i) = val {
                            c.push(*i);
                        }
                    }
                    c
                }"""

content = content.replace("                _ => Vec::new(),", query_state_arm + "\n                _ => Vec::new(),")

with open('crates/vm/src/heap.rs', 'w') as f:
    f.write(content)
