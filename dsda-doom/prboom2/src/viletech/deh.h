#pragma once

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#if !defined(RATBOOM_ZIG) // If included by Zig, don't expand to anything.

struct player_s;
struct pspdef_s;
struct CCore;

// DeHackEd action pointers, general ///////////////////////////////////////////

void A_ClearRefire(struct CCore*, struct player_s*, struct pspdef_s*);

void A_Light(struct CCore*, struct player_s*, struct pspdef_s*);

void A_LightRandomRange(struct CCore*, struct player_s*, struct pspdef_s*);

void A_WeaponProjectileSpread(struct CCore*, struct player_s*, struct pspdef_s*);

void A_WeaponSoundLoop(struct CCore*, struct player_s*, struct pspdef_s*);

void A_WeaponSoundRandom(struct CCore*, struct player_s*, struct pspdef_s*);

// DeHackEd action pointers, FD4RB /////////////////////////////////////////////

void A_BorstalShotgunCheckOverloaded(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BorstalShotgunCheckReload(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BorstalShotgunClearOverload(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BorstalShotgunOverload(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BorstalShotgunReload(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BorstalShotgunDischarge(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BurstShotgunFire(struct CCore*, struct player_s*, struct pspdef_s*);

void A_BurstShotgunCheckVent(struct CCore*, struct player_s*, struct pspdef_s*);

void A_RevolverCheckReload(struct CCore*, struct player_s*, struct pspdef_s*);

#endif // if !defined(RATBOOM_ZIG)
