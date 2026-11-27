//! ORAS's field, what it has that X does not, each handed to the gen 6 field reader through the
//! game's `Game` (`gen6.game`): the DexNav's hidden Pokémon (`hidden`), Secret Bases
//! (`secret_base`), berry spots (`berries`), Route 113's ash piles (`ash`), Fortree Gym's rotating
//! doors (`doors`) and Sootopolis Gym's thin ice (`ice`), its own landforms, and soaring on Latios
//! or Latias (`sky`).

pub mod ash;
pub mod berries;
pub mod doors;
pub mod hidden;
pub mod ice;
pub mod secret_base;
pub mod sky;
