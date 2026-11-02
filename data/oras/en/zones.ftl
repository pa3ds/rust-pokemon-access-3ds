# The parts of a place the game splits into zones on one map, by zone, each a name of pa3ds's own
# with the game's name for the place in it, `$place`, so a way out of the Weather Institute says
# "way out to Northern Route 119" (ADR-0042; Orion chose them on 2026-10-31). A zone of a place in
# parts with no other name of its own says the place alone, as Route 120's main stretch does.
#
# Route 119: the strips north and south of the bridge's west end.
part-0x2e = Northern { $place }
part-0x2f = Southern { $place }
# Route 120: the main stretch, an L, and the raised plateau with its pond at the south-west.
part-0x30 = { $place }
part-0x31 = { $place } Plateau
# The Safari Zone's four quarters, as Serebii names them.
part-0xdb = { $place } Northwest
part-0xdc = { $place } Northeast
part-0xdd = { $place } Southwest
part-0xde = { $place } Southeast
# Route 123's hill, four berry plots parted by paths.
part-0x1dd = { $place } Northwest Berry Plot
part-0x211 = { $place } Northeast Berry Plot
part-0x212 = { $place } Southwest Berry Plot
part-0x213 = { $place } Southeast Berry Plot
