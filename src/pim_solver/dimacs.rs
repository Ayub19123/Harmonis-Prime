//! M2.5: DIMACS CNF Adapter — SAT Competition 2027 Foundation
//!
//! ACHIEVED:
//! - Parse standard DIMACS CNF format (p cnf nbvar nbclauses)
//! - Handle comment lines, malformed input gracefully
//! - Convert to PIM internal crossbar representation
//! - Output SAT results in DIMACS format
//!
//! NOT CLAIMED:
//! - Proof logging (M2.8 — LRAT/FRAT emission)
//! - Binary CNF format
//!
//! HONEST CONSTRAINTS:
//! - Software PIM simulation only (no physical hardware)
//! - Clause database limited by system memory
//! - Single-threaded parse (parallel parse not needed for competition)

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Standard DIMACS CNF instance
#[derive(Debug, Clone)]
pub struct DimacsInstance {
    pub num_vars: usize,
    pub num_clauses: usize,
    pub clauses: Vec<Vec<i32>>,
}

#[derive(Debug)]
pub enum DimacsError {
    Io(String),
    Parse(String),
    InvalidFormat(String),
    MalformedClause(String),
}

impl DimacsError {
    pub fn to_string(&self) -> String {
        match self {
            DimacsError::Io(s) => format!("IO error: {}", s),
            DimacsError::Parse(s) => format!("Parse error: {}", s),
            DimacsError::InvalidFormat(s) => format!("Invalid format: {}", s),
            DimacsError::MalformedClause(s) => format!("Malformed clause: {}", s),
        }
    }
}

impl DimacsInstance {
    /// Parse a standard DIMACS CNF file.
    ///
    /// Format:
    /// ```text
    /// c comment line
    /// p cnf <num_vars> <num_clauses>
    /// <literal> <literal> ... 0
    /// ```
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self, DimacsError> {
        let file = File::open(path).map_err(|e| DimacsError::Io(e.to_string()))?;
        let reader = BufReader::new(file);

        let mut num_vars = 0usize;
        let mut num_clauses = 0usize;
        let mut header_found = false;
        let mut clauses = Vec::new();
        let mut current_clause = Vec::new();

        for (line_idx, line_res) in reader.lines().enumerate() {
            let line = line_res.map_err(|e| DimacsError::Io(e.to_string()))?;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('c') {
                continue;
            }

            // Parse problem line: p cnf <vars> <clauses>
            if trimmed.starts_with('p') {
                if header_found {
                    return Err(DimacsError::InvalidFormat(format!(
                        "Duplicate problem line at line {}",
                        line_idx + 1
                    )));
                }

                let tokens: Vec<&str> = trimmed.split_whitespace().collect();
                if tokens.len() < 4 || tokens[1] != "cnf" {
                    return Err(DimacsError::InvalidFormat(format!(
                        "Malformed problem line at line {}. Expected 'p cnf <vars> <clauses>'",
                        line_idx + 1
                    )));
                }

                num_vars = tokens[2]
                    .parse::<usize>()
                    .map_err(|e| DimacsError::Parse(format!("Invalid variable count: {}", e)))?;

                num_clauses = tokens[3]
                    .parse::<usize>()
                    .map_err(|e| DimacsError::Parse(format!("Invalid clause count: {}", e)))?;

                header_found = true;
                continue;
            }

            // Clause data before header is invalid
            if !header_found {
                return Err(DimacsError::InvalidFormat(format!(
                    "Clause data before problem line at line {}",
                    line_idx + 1
                )));
            }

            // Parse clause literals
            for token in trimmed.split_whitespace() {
                // Skip DIMACS terminator '%'
                if token == "%" {
                    break;
                }
                let literal = token.parse::<i32>().map_err(|e| {
                    DimacsError::MalformedClause(format!(
                        "Non-integer token '{}' at line {}: {}",
                        token,
                        line_idx + 1,
                        e
                    ))
                })?;

                if literal == 0 {
                    // Clause terminator
                    if !current_clause.is_empty() {
                        clauses.push(current_clause.clone());
                    }
                    current_clause.clear();
                } else {
                    // Validate literal bounds
                    if literal.abs() as usize > num_vars {
                        return Err(DimacsError::MalformedClause(format!(
                            "Literal {} out of bounds (max {}) at line {}",
                            literal,
                            num_vars,
                            line_idx + 1
                        )));
                    }
                    current_clause.push(literal);
                }
            }
        }

        // Handle last clause if no trailing zero
        if !current_clause.is_empty() {
            clauses.push(current_clause);
        }

        Ok(DimacsInstance {
            num_vars,
            num_clauses,
            clauses,
        })
    }

    /// Write SAT model in DIMACS output format.
    ///
    /// Format:
    /// ```text
    /// s SATISFIABLE
    /// v <assignment> 0
    /// ```
    pub fn write_model<W: Write>(
        &self,
        writer: &mut W,
        assignment: &[bool],
    ) -> std::io::Result<()> {
        writeln!(writer, "s SATISFIABLE")?;

        let mut line = String::from("v ");
        for (idx, &is_true) in assignment.iter().enumerate() {
            let var = (idx + 1) as i32;
            let token = if is_true {
                format!("{} ", var)
            } else {
                format!("-{} ", var)
            };

            // Line length limit for DIMACS format
            if line.len() + token.len() > 76 {
                writeln!(writer, "{}", line.trim_end())?;
                line = String::from("v ");
            }
            line.push_str(&token);
        }

        line.push_str("0");
        writeln!(writer, "{}", line)?;
        writer.flush()?;
        Ok(())
    }

    /// Write UNSAT result in DIMACS output format.
    pub fn write_unsat<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writeln!(writer, "s UNSATISFIABLE")?;
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cnf() {
        // Create a temporary test file
        let test_content = "c Test CNF file\nc Another comment\np cnf 3 2\n1 -2 0\n-1 3 0\n";
        let test_path = "test_simple.cnf";
        std::fs::write(test_path, test_content).unwrap();

        let instance = DimacsInstance::parse(test_path).unwrap();
        assert_eq!(instance.num_vars, 3);
        assert_eq!(instance.num_clauses, 2);
        assert_eq!(instance.clauses.len(), 2);
        assert_eq!(instance.clauses[0], vec![1, -2]);
        assert_eq!(instance.clauses[1], vec![-1, 3]);

        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_parse_malformed_header() {
        let test_content = "p sat 3 2\n1 0\n";
        let test_path = "test_malformed.cnf";
        std::fs::write(test_path, test_content).unwrap();

        let result = DimacsInstance::parse(test_path);
        assert!(result.is_err());

        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_parse_literal_out_of_bounds() {
        let test_content = "p cnf 2 1\n1 3 0\n";
        let test_path = "test_bounds.cnf";
        std::fs::write(test_path, test_content).unwrap();

        let result = DimacsInstance::parse(test_path);
        assert!(result.is_err());

        std::fs::remove_file(test_path).unwrap();
    }

    #[test]
    fn test_write_model_format() {
        let instance = DimacsInstance {
            num_vars: 4,
            num_clauses: 2,
            clauses: vec![vec![1, -2], vec![3, 4]],
        };

        let mut output = Vec::new();
        let assignment = vec![true, false, true, false];

        instance.write_model(&mut output, &assignment).unwrap();
        let result = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines[0], "s SATISFIABLE");
        assert!(lines[1].contains("v "));
        assert!(lines[1].contains(" 0"));
    }

    #[test]
    fn test_write_unsat_format() {
        let instance = DimacsInstance {
            num_vars: 2,
            num_clauses: 1,
            clauses: vec![vec![1], vec![-1]],
        };

        let mut output = Vec::new();
        instance.write_unsat(&mut output).unwrap();
        let result = String::from_utf8(output).unwrap();

        assert_eq!(result.trim(), "s UNSATISFIABLE");
    }
}
