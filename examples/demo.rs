use nanocolor::{style, Colorize};

fn main() {
    // Foreground colors
    println!("\n{}", "Standard colors:".bold().underline());
    println!(
        "  {} {} {} {} {} {} {} {}",
        "black".black(),
        "red".red(),
        "green".green(),
        "yellow".yellow(),
        "blue".blue(),
        "magenta".magenta(),
        "cyan".cyan(),
        "white".white(),
    );
    println!(
        "  {} {} {} {} {} {} {} {}",
        "bright black".bright_black(),
        "bright red".bright_red(),
        "bright green".bright_green(),
        "bright yellow".bright_yellow(),
        "bright blue".bright_blue(),
        "bright magenta".bright_magenta(),
        "bright cyan".bright_cyan(),
        "bright white".bright_white(),
    );

    // Background colors
    println!("\n{}", "Background colors:".bold().underline());
    println!(
        "  {} {} {} {}",
        " on_red ".on_red(),
        " on_green ".on_green(),
        " on_blue ".on_blue(),
        " on_yellow ".on_yellow(),
    );

    // Text styles
    println!("\n{}", "Text styles:".bold().underline());
    println!("  {}", "bold".bold());
    println!("  {}", "dim".dim());
    println!("  {}", "italic".italic());
    println!("  {}", "underline".underline());
    println!("  {}", "strikethrough".strikethrough());
    println!("  {}", "blink".blink());
    println!("  {}", "reverse".reverse());
    println!("  {}", "hidden (invisible)".hidden());
    println!("  {}", "overline".overline());

    // Chaining
    println!("\n{}", "Chaining:".bold().underline());
    println!("  {}", "red bold underline".red().bold().underline());
    println!("  {}", "green on white italic".green().on_white().italic());
    println!(
        "  {}",
        "bright cyan dim strikethrough"
            .bright_cyan()
            .dim()
            .strikethrough()
    );

    // Primitive types
    println!("\n{}", "Primitive types:".bold().underline());
    println!("  integer: {}", 42.red().bold());
    println!("  float:   {}", 3.14_f64.green());
    println!("  bool:    {}", true.cyan().italic());
    println!("  char:    {}", '✓'.bright_green().bold());

    // style() helper
    println!("\n{}", "style() helper:".bold().underline());
    println!("  {}", style(format!("v{}.{}.{}", 0, 2, 0)).yellow().bold());

    // Conditional styling with .whenever()
    println!("\n{}", "Conditional styling:".bold().underline());
    let verbose = true;
    println!(
        "  whenever(true):  {}",
        "styled".red().bold().whenever(true)
    );
    println!(
        "  whenever(false): {}",
        "not styled".red().bold().whenever(false)
    );
    println!(
        "  whenever(cond):  {}",
        "conditional".yellow().whenever(verbose)
    );

    // Decorative masking
    println!("\n{}", "Decorative masking:".bold().underline());
    println!("  with colors: {}done", "✓ ".green().bold().mask());
    println!(
        "  masked off:  {}done",
        "✓ ".green().bold().mask().whenever(false)
    );
    println!();
}
