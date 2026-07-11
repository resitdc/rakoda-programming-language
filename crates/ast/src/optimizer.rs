use crate::{Expression, Statement, Program, InfixOperator, PrefixOperator};

pub fn optimize_program(mut program: Program) -> Program {
    let mut optimized_statements = Vec::new();
    for stmt in program.statements {
        optimized_statements.push(optimize_statement(stmt));
    }
    program.statements = optimized_statements;
    program
}

fn optimize_statement(stmt: Statement) -> Statement {
    match stmt {
        Statement::DeklarasiVariabel { nama, nilai, lokasi } => Statement::DeklarasiVariabel {
            nama,
            nilai: optimize_expression(nilai),
            lokasi,
        },
        Statement::Assignment { nama, nilai, lokasi } => Statement::Assignment {
            nama,
            nilai: optimize_expression(nilai),
            lokasi,
        },
        Statement::Tampilkan { mut nilai, lokasi } => {
            for i in 0..nilai.len() {
                nilai[i] = optimize_expression(nilai[i].clone());
            }
            Statement::Tampilkan { nilai, lokasi }
        }
        Statement::Cetak { mut nilai, lokasi } => {
            for i in 0..nilai.len() {
                nilai[i] = optimize_expression(nilai[i].clone());
            }
            Statement::Cetak { nilai, lokasi }
        }
        Statement::Kembalikan { nilai, lokasi } => {
            let optimized_nilai = nilai.map(optimize_expression);
            Statement::Kembalikan {
                nilai: optimized_nilai,
                lokasi,
            }
        }
        Statement::Expression(expr) => Statement::Expression(optimize_expression(expr)),
        Statement::Jika {
            kondisi,
            konsekuensi,
            alternatif,
            lokasi,
        } => {
            let opt_kondisi = optimize_expression(kondisi);
            let opt_konsekuensi = konsekuensi.into_iter().map(optimize_statement).collect();
            let opt_alternatif = alternatif.map(|alt| alt.into_iter().map(optimize_statement).collect());
            Statement::Jika {
                kondisi: opt_kondisi,
                konsekuensi: opt_konsekuensi,
                alternatif: opt_alternatif,
                lokasi,
            }
        }
        Statement::Selama { kondisi, body, lokasi } => {
            let opt_kondisi = optimize_expression(kondisi);
            let opt_body = body.into_iter().map(optimize_statement).collect();
            Statement::Selama {
                kondisi: opt_kondisi,
                body: opt_body,
                lokasi,
            }
        }
        Statement::DeklarasiFungsi {
            nama,
            parameter,
            body,
            lokasi,
        } => {
            let opt_body = body.into_iter().map(optimize_statement).collect();
            Statement::DeklarasiFungsi {
                nama,
                parameter,
                body: opt_body,
                lokasi,
            }
        }
    }
}

fn optimize_expression(expr: Expression) -> Expression {
    match expr {
        Expression::Infix { kiri, operator, kanan, lokasi } => {
            let opt_kiri = optimize_expression(*kiri);
            let opt_kanan = optimize_expression(*kanan);

            match (&opt_kiri, &opt_kanan) {
                (Expression::Angka(k, _), Expression::Angka(kn, _)) => {
                    match operator {
                        InfixOperator::Tambah => Expression::Angka(k + kn, lokasi),
                        InfixOperator::Kurang => Expression::Angka(k - kn, lokasi),
                        InfixOperator::Kali => Expression::Angka(k * kn, lokasi),
                        InfixOperator::Bagi => {
                            if *kn != 0.0 {
                                Expression::Angka(k / kn, lokasi)
                            } else {
                                Expression::Infix { kiri: Box::new(opt_kiri), operator, kanan: Box::new(opt_kanan), lokasi }
                            }
                        },
                        InfixOperator::Mod => Expression::Angka(k % kn, lokasi),
                        InfixOperator::LebihDari => Expression::Boolean(k > kn, lokasi),
                        InfixOperator::KurangDari => Expression::Boolean(k < kn, lokasi),
                        InfixOperator::Minimal => Expression::Boolean(k >= kn, lokasi),
                        InfixOperator::Maksimal => Expression::Boolean(k <= kn, lokasi),
                        InfixOperator::SamaDengan => Expression::Boolean(k == kn, lokasi),
                        InfixOperator::TidakSamaDengan => Expression::Boolean(k != kn, lokasi),
                        _ => Expression::Infix { kiri: Box::new(opt_kiri), operator, kanan: Box::new(opt_kanan), lokasi }
                    }
                }
                (Expression::String(k, _), Expression::String(kn, _)) => {
                    match operator {
                        InfixOperator::Tambah => Expression::String(format!("{}{}", k, kn), lokasi),
                        InfixOperator::SamaDengan => Expression::Boolean(k == kn, lokasi),
                        InfixOperator::TidakSamaDengan => Expression::Boolean(k != kn, lokasi),
                        _ => Expression::Infix { kiri: Box::new(opt_kiri), operator, kanan: Box::new(opt_kanan), lokasi }
                    }
                }
                _ => Expression::Infix { kiri: Box::new(opt_kiri), operator, kanan: Box::new(opt_kanan), lokasi }
            }
        }
        Expression::Prefix { operator, kanan, lokasi } => {
            let opt_kanan = optimize_expression(*kanan);
            match (&operator, &opt_kanan) {
                (PrefixOperator::Minus, Expression::Angka(val, _)) => Expression::Angka(-val, lokasi),
                (PrefixOperator::Bukan, Expression::Boolean(val, _)) => Expression::Boolean(!val, lokasi),
                _ => Expression::Prefix { operator, kanan: Box::new(opt_kanan), lokasi }
            }
        }
        Expression::Call { fungsi, argumen, lokasi } => {
            let opt_fungsi = optimize_expression(*fungsi);
            let opt_argumen = argumen.into_iter().map(optimize_expression).collect();
            Expression::Call {
                fungsi: Box::new(opt_fungsi),
                argumen: opt_argumen,
                lokasi,
            }
        }
        Expression::Array { elemen, lokasi } => {
            let opt_elemen = elemen.into_iter().map(optimize_expression).collect();
            Expression::Array { elemen: opt_elemen, lokasi }
        }
        Expression::Kamus { pasangan, lokasi } => {
            let opt_pasangan = pasangan.into_iter().map(|(k, v)| (k, optimize_expression(v))).collect();
            Expression::Kamus { pasangan: opt_pasangan, lokasi }
        }
        Expression::Index { kiri, indeks, lokasi } => {
            let opt_kiri = optimize_expression(*kiri);
            let opt_indeks = optimize_expression(*indeks);
            Expression::Index { kiri: Box::new(opt_kiri), indeks: Box::new(opt_indeks), lokasi }
        }
        // Base cases
        _ => expr
    }
}
