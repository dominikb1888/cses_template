/// DO NOT CHANGE FUNCTION NAME, INPUT OR OUTPUT,
/// WRITES TO STDOUT
fn process<W: std::io::Write>(_input: &str, _writer: &mut W) {
    todo()!;
}

///
/// DO CHANGE CODE BELOW THIS LINE
///
fn main() {
    // Read from stdin and write to stdout
    let input = std::io::read_to_string(std::io::stdin()).unwrap();
    let mut buffer = Vec::new();
    process(&input, &mut buffer);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Include the dynamically generated test code
    include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
}
