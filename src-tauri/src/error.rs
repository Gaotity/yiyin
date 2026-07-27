use crate::dto::CommandErrorDto;

pub type CommandResult<T> = Result<T, CommandErrorDto>;
