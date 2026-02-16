In order

- [x] Main menu
- [x] Game over / Score menu
- [x] Pause menu
- [x] Win condition
- [x] UI ( Current score, time, and collected targets )
  - [x] Center timer correctly
- [x] Player model
- [x] Orient mice to movement direction
- [x] Gravity
  - [x] Check if the player is below ground and make them respawn
- [x] Target model (mice atm)
- [x] Load transitions
- [x] Tweak transition animation
- [ ] Better UI
  - [x] Background
  - [ ] Icons
  - [x] Juice
    - [x] Timer gets red when low on time (10 sec)
    - [x] Target counter pops when picking up a target
    - [x] Change timer font size instead of scaling the node
  - [x] Goal message with fade off
- [ ] Better Menu
  - [x] Proper spacing
  - [ ] Popup on hover
- [x] Power ups
- [x] Lights definition per levels
- [x] Refactor Pickup and Target Hit with Events
- [ ] SFX
  - [x] Target pickup
  - [x] Powerup pickup
  - [x] Jump
  - [ ] Laser
  - [x] Menu button
- [x] Replace Powerup model
- [ ] Finish level 1
- [x] Refactor Transition (so we can use it elsewhere)
- [ ] Better Game over
  - [ ] Laser
  - [ ] Transition
- [ ] Level 2 (Necromancer)
- [x] Music
- [x] Main menu background
- [x] Score menu background
- [ ] Level 3 (Ducks)
- [ ] Level 4 (Crabs)
- [x] Jump
- [ ] Better Target AI
- [x] Adjust shadow settings
- [ ] Credits menu
- [ ] Change rat tail to physics joints
- [ ] Camera collisions / physics

FIXES:
- [x] Normals on cheese
- [x] Player head follows Y velocity
- [x] Camera jump on load
- [ ] Better target goal pick algo
    - right now if the goal is on the edge of the navmesh, the targets will lock up most of the time
- [ ] Transition at the end of a won round skip the transition in

BUGS:
- [x] Something is wrong with how velocity is calculated / apply / interacts with controller
- [x] Sometime targets don't spawn
- [x] Targets spawn "inside" collisions (make collision solid so the there is no navmeshes inside the collisions)
  - Check for stuck targets and have them respawn?
- [x] Restarting a game after loosing make the game crash
  - The game needs a proper reset procedure
- [ ] Sometimes targets don't move (for longer than their idle time)
- [x] Powerup moves it's collider
- [ ] Verlet chain stop updating when player is not moving

REFACTOR:
- [ ] On<SceneInstanceReady>
- [ ] children.iter_descendants(entity)
