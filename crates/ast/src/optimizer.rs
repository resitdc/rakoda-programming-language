use crate::{Expression, InfixOperator, PrefixOperator, Program, Statement};

pub fn optimize_program(mut program: Program) -> Program {
    let mut optimized_statements = Vec::new();
    for stmt in program.statements {
        optimized_statements.extend(optimize_statement(stmt));
    }
    program.statements = optimized_statements;
    program
}

fn optimize_statement(stmt: Statement) -> Vec<Statement> {
    match stmt {
        Statement::DeklarasiVariabel {
            nama,
            nilai,
            lokasi,
        } => vec![Statement::DeklarasiVariabel {
            nama,
            nilai: optimize_expression(nilai),
            lokasi,
        }],
        Statement::Assignment {
            nama,
            nilai,
            lokasi,
        } => vec![Statement::Assignment {
            nama,
            nilai: optimize_expression(nilai),
            lokasi,
        }],
        Statement::Tampilkan { nilai, lokasi } => {
            let nilai = nilai.into_iter().map(optimize_expression).collect();
            vec![Statement::Tampilkan { nilai, lokasi }]
        }
        Statement::Cetak { nilai, lokasi } => {
            let nilai = nilai.into_iter().map(optimize_expression).collect();
            vec![Statement::Cetak { nilai, lokasi }]
        }
        Statement::Kembalikan { nilai, lokasi } => {
            let optimized_nilai = nilai.map(optimize_expression);
            vec![Statement::Kembalikan {
                nilai: optimized_nilai,
                lokasi,
            }]
        }
        Statement::Expression(expr) => vec![Statement::Expression(optimize_expression(expr))],
        Statement::Jika {
            kondisi,
            konsekuensi,
            alternatif,
            lokasi,
        } => {
            let opt_kondisi = optimize_expression(kondisi);

            // Dead Code Elimination
            if let Expression::Boolean(b, _) = opt_kondisi {
                if b {
                    // kondisi selalu benar, buang Jika, kembalikan isi konsekuensi
                    let mut inlined = Vec::new();
                    for stmt in konsekuensi {
                        inlined.extend(optimize_statement(stmt));
                    }
                    return inlined;
                } else {
                    // kondisi selalu salah, buang Jika, kembalikan alternatif (jika ada)
                    let mut inlined = Vec::new();
                    if let Some(alt) = alternatif {
                        for stmt in alt {
                            inlined.extend(optimize_statement(stmt));
                        }
                    }
                    return inlined;
                }
            }

            let opt_konsekuensi = konsekuensi
                .into_iter()
                .flat_map(optimize_statement)
                .collect();
            let opt_alternatif =
                alternatif.map(|alt| alt.into_iter().flat_map(optimize_statement).collect());
            vec![Statement::Jika {
                kondisi: opt_kondisi,
                konsekuensi: opt_konsekuensi,
                alternatif: opt_alternatif,
                lokasi,
            }]
        }
        Statement::Selama {
            kondisi,
            body,
            lokasi,
        } => {
            let opt_kondisi = optimize_expression(kondisi);

            if let Expression::Boolean(false, _) = opt_kondisi {
                // Selama (salah) { ... } tidak akan pernah jalan
                return vec![];
            }

            let opt_body = body.into_iter().flat_map(optimize_statement).collect();
            vec![Statement::Selama {
                kondisi: opt_kondisi,
                body: opt_body,
                lokasi,
            }]
        }
        Statement::Setiap {
            elemen,
            koleksi,
            indeks,
            body,
            lokasi,
        } => {
            let opt_koleksi = optimize_expression(koleksi);
            let opt_body = body.into_iter().flat_map(optimize_statement).collect();
            vec![Statement::Setiap {
                elemen,
                koleksi: opt_koleksi,
                indeks,
                body: opt_body,
                lokasi,
            }]
        }
        Statement::DeklarasiFungsi {
            nama,
            parameter,
            body,
            lokasi,
        } => {
            let opt_body = body.into_iter().flat_map(optimize_statement).collect();
            vec![Statement::DeklarasiFungsi {
                nama,
                parameter,
                body: opt_body,
                lokasi,
            }]
        }
        Statement::CobaTangkap {
            coba_body,
            error_ident,
            tangkap_body,
            lokasi,
        } => {
            let opt_coba = coba_body.into_iter().flat_map(optimize_statement).collect();
            let opt_tangkap = tangkap_body
                .into_iter()
                .flat_map(optimize_statement)
                .collect();
            vec![Statement::CobaTangkap {
                coba_body: opt_coba,
                error_ident,
                tangkap_body: opt_tangkap,
                lokasi,
            }]
        }
        Statement::Lempar { nilai, lokasi } => {
            vec![Statement::Lempar {
                nilai: optimize_expression(nilai),
                lokasi,
            }]
        }
        Statement::Error(lokasi) => {
            vec![Statement::Error(lokasi)]
        }
    }
}

fn optimize_expression(expr: Expression) -> Expression {
    match expr {
        Expression::Infix {
            kiri,
            operator,
            kanan,
            lokasi,
        } => {
            let opt_kiri = optimize_expression(*kiri);
            let opt_kanan = optimize_expression(*kanan);

            match (&opt_kiri, &opt_kanan) {
                (Expression::Angka(k, _), Expression::Angka(kn, _)) => match operator {
                    InfixOperator::Tambah => Expression::Angka(k + kn, lokasi),
                    InfixOperator::Kurang => Expression::Angka(k - kn, lokasi),
                    InfixOperator::Kali => Expression::Angka(k * kn, lokasi),
                    InfixOperator::Bagi => {
                        if *kn != 0.0 {
                            Expression::Angka(k / kn, lokasi)
                        } else {
                            Expression::Infix {
                                kiri: Box::new(opt_kiri),
                                operator,
                                kanan: Box::new(opt_kanan),
                                lokasi,
                            }
                        }
                    }
                    InfixOperator::Mod => Expression::Angka(k % kn, lokasi),
                    InfixOperator::LebihDari => Expression::Boolean(k > kn, lokasi),
                    InfixOperator::KurangDari => Expression::Boolean(k < kn, lokasi),
                    InfixOperator::Minimal => Expression::Boolean(k >= kn, lokasi),
                    InfixOperator::Maksimal => Expression::Boolean(k <= kn, lokasi),
                    InfixOperator::SamaDengan => Expression::Boolean(k == kn, lokasi),
                    InfixOperator::TidakSamaDengan => Expression::Boolean(k != kn, lokasi),
                    _ => Expression::Infix {
                        kiri: Box::new(opt_kiri),
                        operator,
                        kanan: Box::new(opt_kanan),
                        lokasi,
                    },
                },
                (Expression::String(k, _), Expression::String(kn, _)) => match operator {
                    InfixOperator::Tambah => Expression::String(format!("{}{}", k, kn), lokasi),
                    InfixOperator::SamaDengan => Expression::Boolean(k == kn, lokasi),
                    InfixOperator::TidakSamaDengan => Expression::Boolean(k != kn, lokasi),
                    _ => Expression::Infix {
                        kiri: Box::new(opt_kiri),
                        operator,
                        kanan: Box::new(opt_kanan),
                        lokasi,
                    },
                },
                _ => Expression::Infix {
                    kiri: Box::new(opt_kiri),
                    operator,
                    kanan: Box::new(opt_kanan),
                    lokasi,
                },
            }
        }
        Expression::Prefix {
            operator,
            kanan,
            lokasi,
        } => {
            let opt_kanan = optimize_expression(*kanan);
            match (&operator, &opt_kanan) {
                (PrefixOperator::Minus, Expression::Angka(val, _)) => {
                    Expression::Angka(-val, lokasi)
                }
                (PrefixOperator::Bukan, Expression::Boolean(val, _)) => {
                    Expression::Boolean(!val, lokasi)
                }
                _ => Expression::Prefix {
                    operator,
                    kanan: Box::new(opt_kanan),
                    lokasi,
                },
            }
        }
        Expression::Call {
            fungsi,
            argumen,
            lokasi,
        } => {
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
            Expression::Array {
                elemen: opt_elemen,
                lokasi,
            }
        }
        Expression::Kamus { pasangan, lokasi } => {
            let opt_pasangan = pasangan
                .into_iter()
                .map(|(k, v)| (k, optimize_expression(v)))
                .collect();
            Expression::Kamus {
                pasangan: opt_pasangan,
                lokasi,
            }
        }
        Expression::Index {
            kiri,
            indeks,
            lokasi,
        } => {
            let opt_kiri = optimize_expression(*kiri);
            let opt_indeks = optimize_expression(*indeks);
            Expression::Index {
                kiri: Box::new(opt_kiri),
                indeks: Box::new(opt_indeks),
                lokasi,
            }
        }
        Expression::FungsiAnonim {
            parameter,
            body,
            lokasi,
        } => {
            let opt_body = body.into_iter().flat_map(optimize_statement).collect();
            Expression::FungsiAnonim {
                parameter,
                body: opt_body,
                lokasi,
            }
        }
        // Base cases
        _ => expr,
    }
}
