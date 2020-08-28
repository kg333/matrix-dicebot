use nom::{alt, complete, named, tag, take_while, tuple, IResult};

use crate::cofd::parser::{create_chance_die, parse_dice_pool};
use crate::commands::{Command, HelpCommand, PoolRollCommand, RollCommand};
use crate::dice::parser::parse_element_expression;
use crate::help::parse_help_topic;
use crate::parser::{eat_whitespace, trim};

// Parse a roll expression.
fn parse_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, _) = eat_whitespace(input)?;
    let (input, expression) = parse_element_expression(input)?;
    Ok((input, Box::new(RollCommand(expression))))
}

fn parse_pool_roll(input: &str) -> IResult<&str, Box<dyn Command>> {
    let (input, _) = eat_whitespace(input)?;
    let (input, pool) = parse_dice_pool(input)?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn chance_die() -> IResult<&'static str, Box<dyn Command>> {
    let (input, pool) = create_chance_die()?;
    Ok((input, Box::new(PoolRollCommand(pool))))
}

fn help(topic: &str) -> IResult<&str, Box<dyn Command>> {
    let (topic, _) = eat_whitespace(topic)?;
    let topic = parse_help_topic(&trim(topic));
    Ok(("", Box::new(HelpCommand(topic))))
}

/// Potentially parse a command expression.  If we recognize the command, an error should be raised
/// if the command is misparsed.  If we don't recognize the command, ignore it and return none
pub fn parse_command(original_input: &str) -> IResult<&str, Option<Box<dyn Command>>> {
    let (input, _) = eat_whitespace(original_input)?;

    //Parser understands either specific !commands with no input, or any !command with extra input.
    named!(command(&str) -> (&str, &str), tuple!(
        complete!(tag!("!")),
        alt!(
            //TODO figure out how to gracefully handle arbitrary single commands.
            complete!(tag!("chance")) |
            complete!(tag!("help")) |
            complete!(take_while!(char::is_alphabetic))
        )
    ));

    let (input, command) = match command(input) {
        // Strip the exclamation mark
        Ok((input, (_, result))) => (input, result),
        Err(_e) => {
            return Ok((original_input, None));
        }
    };

    match command {
        "r" | "roll" => parse_roll(input).map(|(input, command)| (input, Some(command))),
        "rp" | "pool" => parse_pool_roll(input).map(|(input, command)| (input, Some(command))),
        "chance" => chance_die().map(|(input, command)| (input, Some(command))),
        "help" => help(input).map(|(input, command)| (input, Some(command))),
        // No recognized command, ignore this.
        _ => Ok((original_input, None)),
    }
}
