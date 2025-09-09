/// Calculates the maximum width needed for each column in a table
pub fn calculate_column_widths(table: &Vec<Vec<String>>) -> Vec<usize> {
    if table.is_empty() {
        return Vec::new();
    }

    let max_cols = table.iter().map(|row| row.len()).max().unwrap_or(0);
    let mut col_widths = vec![0; max_cols];

    for row in table {
        for (i, cell) in row.iter().enumerate() {
            if i < max_cols {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    col_widths
}

pub fn print_markdown_table(table: &Vec<Vec<String>>, header_line: bool) {
    // Handle empty table
    if table.is_empty() {
        return;
    }

    // Find the maximum width needed for each column
    let max_cols = table.iter().map(|row| row.len()).max().unwrap_or(0);
    let col_widths = calculate_column_widths(table);

    // Print each row with proper alignment
    for (row_idx, row) in table.iter().enumerate() {
        // Start with a leading pipe
        print!("|");

        for (i, cell) in row.iter().enumerate() {
            if i < max_cols {
                // Right-pad each cell to match the column width
                print!(" {:<width$} ", cell, width = col_widths[i]);

                // Add column separator
                if i < max_cols - 1 {
                    print!("|");
                }
            }
        }

        // End with a trailing pipe
        println!("|");

        // Add separator line after the header if requested
        if row_idx == 0 && header_line && table.len() > 1 {
            print!("|");
            for (i, width) in col_widths.iter().enumerate() {
                print!(":{}", "-".repeat(*width + 1));
                if i < max_cols - 1 {
                    print!("|");
                }
            }
            println!("|");
        }
    }
}

pub fn print_buffer(buffer: &[u8]) {
    // Create table with header
    let mut table = vec![vec![
        "Byte".to_string(),
        "Word".to_string(),
        "00".to_string(),
        "01".to_string(),
        "02".to_string(),
        "03".to_string(),
        "04".to_string(),
        "05".to_string(),
        "06".to_string(),
        "07".to_string(),
        "08".to_string(),
        "09".to_string(),
        "0A".to_string(),
        "0B".to_string(),
        "0C".to_string(),
        "0D".to_string(),
        "0E".to_string(),
        "0F".to_string(),
        "ASCII".to_string(),
    ]];

    // Process buffer in chunks of 16 bytes
    for (chunk_idx, chunk) in buffer.chunks(16).enumerate() {
        let byte_address = chunk_idx * 16;
        let word_address = byte_address / 2;
        let mut row = vec![
            format!("0x{:04X}", byte_address),
            format!("0x{:04X}", word_address),
        ];

        // Add hex values
        for i in 0..16 {
            if i < chunk.len() {
                row.push(format!("{:02X}", chunk[i]));
            } else {
                row.push("".to_string());
            }
        }

        // Add ASCII representation
        let ascii: String = chunk
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        row.push(ascii);

        table.push(row);
    }

    // get max column widths
    let col_widths = calculate_column_widths(&table);

    // Print the table (without pipes, don't use markdown table
    for row in &table {
        for (i, cell) in row.iter().enumerate() {
            if i == 0 {
                print!("{:<width$}", cell, width = col_widths[i]);
            } else {
                print!(" {:<width$}", cell, width = col_widths[i]);
            }
        }
        println!();
    }
}
