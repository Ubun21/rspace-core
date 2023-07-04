use std::str::FromStr;

#[derive(Copy, Clone)]
pub enum ChunkIdAlgo {
  Named,
  Numeric,
}

impl ChunkIdAlgo {
  pub fn is_named(&self) -> bool {
    matches!(self, Self::Named)
  }

  pub fn is_numeric(&self) -> bool {
    matches!(self, Self::Numeric)
  }
}

#[derive(Clone, Copy)]
pub enum ModuleIdAlgo {
  Named,
  Numeric,
}

impl ModuleIdAlgo {
  pub fn is_named(&self) -> bool {
    matches!(self, Self::Named)
  }

  pub fn is_numeric(&self) -> bool {
    matches!(self, Self::Numeric)
  }
}

impl FromStr for ModuleIdAlgo {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "named" => Ok(Self::Named),
      _ => Err(()),
    }
  }
}

impl FromStr for ChunkIdAlgo {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "named" => Ok(Self::Named),
      _ => Err(()),
    }
  }
}
