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
- [ ] Tweak transition animation
- [ ] Better UI
  - [x] Background
  - [ ] Icons
  - [ ] Juice
    - [x] Timer gets red when low on time (10 sec)
    - [ ] Target counter pops when picking up a target
  - [ ] Goal message with fade off
- [ ] Better Menu
  - [x] Proper spacing
  - [ ] Popup on hover
- [x] Power ups
- [ ] Lights definition per levels (From blender?)
- [x] Refactor Pickup and Target Hit with Events
- [ ] SFX
- [ ] Replace Powerup model
- [ ] Finish level 1
- [ ] Better Game over
  - [ ] Laser
  - [ ] Transition
  - [ ] High scores?
- [ ] Level 2 (Necromancer)
- [ ] Music
- [ ] Main menu background
- [ ] Level 3 (Ducks)
- [ ] Level 4 (??)
- [x] Jump
- [ ] Better Target AI
- [ ] Adjust shadow settings
  - CascadeShadowConfigBuilder {
              first_cascade_far_bound: 200.0,
              maximum_distance: 400.0,
              ..default()
          }
- [ ] Change rat tail to physics joints
- [ ] Camera collisions / physics

BUGS:
- [x] Something is wrong with how velocity is calculated / apply / interacts with controller
- [x] Sometime targets don't spawn
- [x] Targets spawn "inside" collisions (make collision solid so the there is no navmeshes inside the collisions)
  - Check for stuck targets and have them respawn?
- [x] Restarting a game after loosing make the game crash
  - The game needs a proper reset procedure
- [ ] Sometimes targets don't move (for longer than their idle time)
- [ ] Powerup moves it's collider
- [ ] Verlet chain stop updating when player is not moving

REFACTOR:
- [ ] On<SceneInstanceReady>
- [ ] children.iter_descendants(entity)
