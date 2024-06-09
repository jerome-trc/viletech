/* Emacs style mode select   -*- C -*-
 *-----------------------------------------------------------------------------
 *
 *
 *  PrBoom: a Doom port merged with LxDoom and LSDLDoom
 *  based on BOOM, a modified and improved DOOM engine
 *  Copyright (C) 1999 by
 *  id Software, Chi Hoang, Lee Killough, Jim Flynn, Rand Phares, Ty Halderman
 *  Copyright (C) 1999-2002 by
 *  Jess Haas, Nicolas Kalkhof, Colin Phipps, Florian Schulze
 *  Copyright 2005, 2006 by
 *  Florian Schulze, Colin Phipps, Neil Stevens, Andrey Budko
 *
 *  This program is free software; you can redistribute it and/or
 *  modify it under the terms of the GNU General Public License
 *  as published by the Free Software Foundation; either version 2
 *  of the License, or (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program; if not, write to the Free Software
 *  Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA
 *  02111-1307, USA.
 *
 * DESCRIPTION:
 *      Cheat sequence checking.
 *
 *-----------------------------------------------------------------------------*/

#include "doomdef.h"
#include "doomstat.h"
#include "am_map.h"
#include "g_game.h"
#include "p_inter.h"
#include "p_tick.h"
#include "m_cheat.h"
#include "r_state.h"
#include "s_sound.h"
#include "sounds.h"
#include "p_map.h"
#include "d_deh.h"  // Ty 03/27/98 - externalized strings
/* cph 2006/07/23 - needs direct access to thinkercap */
#include "p_tick.h"
#include "p_user.h"

#include "heretic/sb_bar.h"

#include "dsda.h"
#include "dsda/configuration.h"
#include "dsda/excmd.h"
#include "dsda/exhud.h"
#include "dsda/features.h"
#include "dsda/input.h"
#include "dsda/map_format.h"
#include "dsda/mapinfo.h"
#include "dsda/messenger.h"
#include "dsda/settings.h"
#include "dsda/skill_info.h"

#define plyr (players+consoleplayer)     /* the console player */

//-----------------------------------------------------------------------------
//
// CHEAT SEQUENCE PACKAGE
//
//-----------------------------------------------------------------------------

static void cheat_behold(CCore*, void*);
static void cheat_choppers(CCore*, void*);
static void cheat_clev(CCore*, void*);
static void cheat_comp(CCore*, void*);
static void cheat_ddt(CCore*, void*);
static void cheat_fa(CCore*, void*);
static void cheat_fast(CCore*, void*);
static void cheat_fly(CCore*, void*);
static void cheat_freeze(CCore*, void*);
static void cheat_friction(CCore*, void*);
static void cheat_god(CCore*, void*);
static void cheat_health(CCore*, void*);
static void cheat_hom(CCore*, void*);
static void cheat_k(CCore*, void*);
static void cheat_kfa(CCore*, void*);
static void cheat_massacre(CCore*, void*);
static void cheat_megaarmour(CCore*, void*);
static void cheat_mus(CCore*, void*);
static void cheat_mypos(CCore*, void*);
static void cheat_noclip(CCore*, void*);
static void cheat_notarget(CCore*, void*);
static void cheat_pitch(CCore*, void*);
static void cheat_pushers(CCore*, void*);
static void cheat_pw(CCore*, void*);
static void cheat_rate(CCore*, void*);
static void cheat_reveal_item(CCore*, void*);
static void cheat_reveal_kill(CCore*, void*);
static void cheat_reveal_secret(CCore*, void*);
static void cheat_smart(CCore*, void*);
static void cheat_tntammo(CCore*, void*);
static void cheat_tntammox(CCore*, void*);
static void cheat_tntkey(CCore*, void*);
static void cheat_tntkeyx(CCore*, void*);
static void cheat_tntkeyxx(CCore*, void*);
static void cheat_tntweap(CCore*, void*);
static void cheat_tntweapx(CCore*, void*);

// heretic
static void cheat_reset_health(CCore*, void*);
static void cheat_tome(CCore*, void*);
static void cheat_chicken(CCore*, void*);
static void cheat_artifact(CCore*, void*);

// hexen
static void cheat_inventory(CCore*, void*);
static void cheat_puzzle(CCore*, void*);
static void cheat_class(CCore*, void*);
static void cheat_init(CCore*, void*);
static void cheat_script(CCore*, void*);

//-----------------------------------------------------------------------------
//
// List of cheat codes, functions, and special argument indicators.
//
// The first argument is the cheat code.
//
// The second argument is its DEH name, or NULL if it's not supported by -deh.
//
// The third argument is a combination of the bitmasks,
// which excludes the cheat during certain modes of play.
//
// The fourth argument is the handler function.
//
// The fifth argument is passed to the handler function if it's non-negative;
// if negative, then its negative indicates the number of extra characters
// expected after the cheat code, which are passed to the handler function
// via a pointer to a buffer (after folding any letters to lowercase).
//
//-----------------------------------------------------------------------------

cheatseq_t cheat[] = {
  CHEAT("idmus",      "Change music",     cht_always, cheat_mus, -2, false),
  CHEAT("idchoppers", "Chainsaw",         not_demo, cheat_choppers, 0, false),
  CHEAT("iddqd",      "God mode",         not_classic_demo,  cheat_god, 0, false),
  CHEAT("idkfa",      "Ammo & Keys",      not_demo, cheat_kfa, 0, false),
  CHEAT("idfa",       "Ammo",             not_demo, cheat_fa, 0, false),
  CHEAT("idspispopd", "No Clipping 1",    not_classic_demo,  cheat_noclip, 0, false),
  CHEAT("idclip",     "No Clipping 2",    not_classic_demo,  cheat_noclip, 0, false),
  CHEAT("idbeholdh",  "Invincibility",    not_demo, cheat_health, 0, false),
  CHEAT("idbeholdm",  "Invincibility",    not_demo, cheat_megaarmour, 0, false),
  CHEAT("idbeholdv",  "Invincibility",    not_demo, cheat_pw, pw_invulnerability, false),
  CHEAT("idbeholds",  "Berserk",          not_demo, cheat_pw, pw_strength, false),
  CHEAT("idbeholdi",  "Invisibility",     not_demo, cheat_pw, pw_invisibility, false),
  CHEAT("idbeholdr",  "Radiation Suit",   not_demo, cheat_pw, pw_ironfeet, false),
  CHEAT("idbeholda",  "Auto-map",         cht_always, cheat_pw, pw_allmap, false),
  CHEAT("idbeholdl",  "Lite-Amp Goggles", cht_always, cheat_pw, pw_infrared, false),
  CHEAT("idbehold",   "BEHOLD menu",      cht_always, cheat_behold, 0, false),
  CHEAT("idclev",     "Level Warp",       not_demo | not_menu, cheat_clev, -2, false),
  CHEAT("idmypos",    NULL,               cht_always, cheat_mypos, 0, false),
  CHEAT("idrate",     "Frame rate",       cht_always, cheat_rate, 0, false),
  // phares
  CHEAT("tntcomp",    NULL,               not_demo, cheat_comp, -2, false),
  // jff 2/01/98 kill all monsters
  CHEAT("tntem",      NULL,               not_demo, cheat_massacre, 0, false),
  // killough 2/07/98: moved from am_map.c
  CHEAT("iddt",       "Map cheat",        cht_always, cheat_ddt, 0, true),
  CHEAT("iddst",      NULL,               cht_always, cheat_reveal_secret, 0, true),
  CHEAT("iddkt",      NULL,               cht_always, cheat_reveal_kill, 0, true),
  CHEAT("iddit",      NULL,               cht_always, cheat_reveal_item, 0, true),
  // killough 2/07/98: HOM autodetector
  CHEAT("tnthom",     NULL,               cht_always, cheat_hom, 0, false),
  // killough 2/16/98: generalized key cheats
  CHEAT("tntkey",     NULL,               not_demo, cheat_tntkey, 0, false),
  CHEAT("tntkeyr",    NULL,               not_demo, cheat_tntkeyx, 0, false),
  CHEAT("tntkeyy",    NULL,               not_demo, cheat_tntkeyx, 0, false),
  CHEAT("tntkeyb",    NULL,               not_demo, cheat_tntkeyx, 0, false),
  CHEAT("tntkeyrc",   NULL,               not_demo, cheat_tntkeyxx, it_redcard, false),
  CHEAT("tntkeyyc",   NULL,               not_demo, cheat_tntkeyxx, it_yellowcard, false),
  CHEAT("tntkeybc",   NULL,               not_demo, cheat_tntkeyxx, it_bluecard, false),
  CHEAT("tntkeyrs",   NULL,               not_demo, cheat_tntkeyxx, it_redskull, false),
  CHEAT("tntkeyys",   NULL,               not_demo, cheat_tntkeyxx, it_yellowskull, false),
  // killough 2/16/98: end generalized keys
  CHEAT("tntkeybs",   NULL,               not_demo, cheat_tntkeyxx, it_blueskull, false),
  // Ty 04/11/98 - Added TNTKA
  CHEAT("tntka",      NULL,               not_demo, cheat_k, 0, false),
  // killough 2/16/98: generalized weapon cheats
  CHEAT("tntweap",    NULL,               not_demo, cheat_tntweap, 0, false),
  CHEAT("tntweap",    NULL,               not_demo, cheat_tntweapx, -1, false),
  CHEAT("tntammo",    NULL,               not_demo, cheat_tntammo, 0, false),
  // killough 2/16/98: end generalized weapons
  CHEAT("tntammo",    NULL,               not_demo, cheat_tntammox, -1, false),
  // killough 2/21/98: smart monster toggle
  CHEAT("tntsmart",   NULL,               not_demo, cheat_smart, 0, false),
  // killough 2/21/98: pitched sound toggle
  CHEAT("tntpitch",   NULL,               cht_always, cheat_pitch, 0, false),
  // killough 2/21/98: reduce RSI injury by adding simpler alias sequences:
  // killough 2/21/98: same as tntammo
  CHEAT("tntamo",     NULL,               not_demo, cheat_tntammo, 0, false),
  // killough 2/21/98: same as tntammo
  CHEAT("tntamo",     NULL,               not_demo, cheat_tntammox, -1, false),
  // killough 3/6/98: -fast toggle
  CHEAT("tntfast",    NULL,               not_demo, cheat_fast, 0, false),
  // phares 3/10/98: toggle variable friction effects
  CHEAT("tntice",     NULL,               not_demo, cheat_friction, 0, false),
  // phares 3/10/98: toggle pushers
  CHEAT("tntpush",    NULL,               not_demo, cheat_pushers, 0, false),

  // [RH] Monsters don't target
  CHEAT("notarget",   NULL,               not_demo, cheat_notarget, 0, false),
  // fly mode is active
  CHEAT("fly",        NULL,               not_demo, cheat_fly, 0, false),

  // heretic
  CHEAT("quicken", NULL, not_classic_demo, cheat_god, 0, false),
  CHEAT("ponce", NULL, not_demo, cheat_reset_health, 0, false),
  CHEAT("kitty", NULL, not_classic_demo, cheat_noclip, 0, false),
  CHEAT("massacre", NULL, not_demo, cheat_massacre, 0, false),
  CHEAT("rambo", NULL, not_demo, cheat_fa, 0, false),
  CHEAT("skel", NULL, not_demo, cheat_k, 0, false),
  CHEAT("gimme", NULL, not_demo, cheat_artifact, -2, false),
  CHEAT("shazam", NULL, not_demo, cheat_tome, 0, false),
  CHEAT("engage", NULL, not_demo | not_menu, cheat_clev, -2, false),
  CHEAT("ravmap", NULL, cht_always, cheat_ddt, 0, true),
  CHEAT("cockadoodledoo", NULL, not_demo, cheat_chicken, 0, false),

  // hexen
  CHEAT("satan", NULL, not_classic_demo, cheat_god, 0, false),
  CHEAT("clubmed", NULL, not_demo, cheat_reset_health, 0, false),
  CHEAT("butcher", NULL, not_demo, cheat_massacre, 0, false),
  CHEAT("nra", NULL, not_demo, cheat_fa, 0, false),
  CHEAT("indiana", NULL, not_demo, cheat_inventory, 0, false),
  CHEAT("locksmith", NULL, not_demo, cheat_k, 0, false),
  CHEAT("sherlock", NULL, not_demo, cheat_puzzle, 0, false),
  CHEAT("casper", NULL, not_classic_demo, cheat_noclip, 0, false),
  CHEAT("shadowcaster", NULL, not_demo, cheat_class, -1, false),
  CHEAT("visit", NULL, not_demo | not_menu, cheat_clev, -2, false),
  CHEAT("init", NULL, not_demo, cheat_init, 0, false),
  CHEAT("puke", NULL, not_demo, cheat_script, -2, false),
  CHEAT("mapsco", NULL, cht_always, cheat_ddt, 0, true),
  CHEAT("deliverance", NULL, not_demo, cheat_chicken, 0, false),

  // end-of-list marker
  {NULL}
};

//-----------------------------------------------------------------------------

static void cheat_mus(CCore* cx, void* v)
{
    char* buf = v;
  int musnum;

  //jff 3/20/98 note: this cheat allowed in netgame/demorecord

  //jff 3/17/98 avoid musnum being negative and crashing
  if (!isdigit(buf[0]) || !isdigit(buf[1]))
    return;

  dsda_AddMessage(cx, s_STSTR_MUS);

  if (gamemode == commercial)
    {
      musnum = mus_runnin + (buf[0] - '0') * 10 + buf[1] -'0' - 1;

      //jff 4/11/98 prevent IDMUS00 in DOOMII and IDMUS36 or greater
      if (musnum < mus_runnin ||  ((buf[0] - '0') * 10 + buf[1] - '0') > 35)
        dsda_AddMessage(cx, s_STSTR_NOMUS);
      else
        {
          S_ChangeMusic(cx, musnum, 1);
          idmusnum = musnum; //jff 3/17/98 remember idmus number for restore
        }
    }
  else
    {
      musnum = mus_e1m1 + (buf[0] - '1') * 9 + (buf[1] - '1');

      //jff 4/11/98 prevent IDMUS0x IDMUSx0 in DOOMI and greater than introa
      if (buf[0] < '1' || buf[1] < '1' || ((buf[0] - '1')*9 + buf[1] - '1') > 31)
        dsda_AddMessage(cx, s_STSTR_NOMUS);
      else
        {
          S_ChangeMusic(cx,  musnum, 1);
          idmusnum = musnum; //jff 3/17/98 remember idmus number for restore
        }
    }
}

// 'choppers' invulnerability & chainsaw
static void cheat_choppers(CCore* cx, void* v) {
    (void)v;
    plyr->weaponowned[wp_chainsaw] = true;
    plyr->powers[pw_invulnerability] = true;
    dsda_AddMessage(cx, s_STSTR_CHOPPERS);
}

void M_CheatGod(CCore* cx)
{
  // dead players are first respawned at the current position
  if (plyr->playerstate == PST_DEAD)
  {
    signed int an;
    mapthing_t mt = {0};

    P_MapStart();
    mt.x = plyr->mo->x;
    mt.y = plyr->mo->y;
    mt.angle = (plyr->mo->angle + ANG45/2)*(uint64_t)45/ANG45;
    mt.type = consoleplayer + 1;
    mt.options = 1; // arbitrary non-zero value
    P_SpawnPlayer(cx, consoleplayer, &mt);

    // spawn a teleport fog
    an = plyr->mo->angle >> ANGLETOFINESHIFT;
    P_SpawnMobj(cx, plyr->mo->x + 20*finecosine[an],
                plyr->mo->y + 20*finesine[an],
                plyr->mo->z + g_telefog_height,
                g_mt_tfog);
    S_StartMobjSound(plyr->mo, g_sfx_revive);
    P_MapEnd();
  }

  plyr->cheats ^= CF_GODMODE;
  if (plyr->cheats & CF_GODMODE)
  {
    if (plyr->mo)
      plyr->mo->health = god_health;  // Ty 03/09/98 - deh

    plyr->health = god_health;
    dsda_AddMessage(cx, s_STSTR_DQDON);
  }
  else
    dsda_AddMessage(cx, s_STSTR_DQDOFF);

  if (raven) SB_Start();
}

/// 'dqd' cheat for toggleable god mode
static void cheat_god(CCore* cx, void* v)
{
  if (demorecording)
  {
    dsda_QueueExCmdGod();
    return;
  }

  M_CheatGod(cx);
}

// CPhipps - new health and armour cheat codes
static void cheat_health(CCore* cx, void* v)
{
    (void)v;

  if (!(plyr->cheats & CF_GODMODE)) {
    if (plyr->mo)
      plyr->mo->health = mega_health;
    plyr->health = mega_health;
    dsda_AddMessage(cx, s_STSTR_BEHOLDX);
  }
}

static void cheat_megaarmour(CCore* cx, void* v) {
    (void)v;
  plyr->armorpoints[ARMOR_ARMOR] = idfa_armor; // Ty 03/09/98 - deh
  plyr->armortype = idfa_armor_class;  // Ty 03/09/98 - deh
  dsda_AddMessage(cx, s_STSTR_BEHOLDX);
}

static void cheat_fa(CCore* cx, void* v) {
    (void)v;

  int i;

  if (hexen)
  {
    for (i = 0; i < NUMARMOR; i++)
    {
        plyr->armorpoints[i] = pclass[plyr->pclass].armor_increment[i];
    }
    for (i = 0; i < HEXEN_NUMWEAPONS; i++)
    {
        plyr->weaponowned[i] = true;
    }
    for (i = 0; i < NUMMANA; i++)
    {
        plyr->ammo[i] = MAX_MANA;
    }
  }
  else
  {
    if (!plyr->backpack)
    {
      for (i=0 ; i<NUMAMMO ; i++)
        plyr->maxammo[i] *= 2;
      plyr->backpack = true;
    }

    plyr->armorpoints[ARMOR_ARMOR] = idfa_armor; // Ty 03/09/98 - deh
    plyr->armortype = idfa_armor_class;  // Ty 03/09/98 - deh

    // You can't own weapons that aren't in the game // phares 02/27/98
    for (i=0;i<NUMWEAPONS;i++)
      if (!(((i == wp_plasma || i == wp_bfg) && gamemode == shareware) ||
            (i == wp_supershotgun && gamemode != commercial)))
        plyr->weaponowned[i] = true;

    for (i=0;i<NUMAMMO;i++)
      if (i!=am_cell || gamemode!=shareware)
        plyr->ammo[i] = plyr->maxammo[i];

    dsda_AddMessage(cx, s_STSTR_FAADDED);
  }
}

static void cheat_k(CCore* cx, void* v) {
    (void)v;

  int i;
  for (i=0;i<NUMCARDS;i++)
    if (!plyr->cards[i])     // only print message if at least one key added
      {                      // however, caller may overwrite message anyway
        plyr->cards[i] = true;
        dsda_AddMessage(cx, "Keys Added");
      }

  // heretic - reset status bar
  SB_Start();
}

static void cheat_kfa(CCore* cx, void* v)
{
  cheat_k(cx, v);
  cheat_fa(cx, v);
  dsda_AddMessage(cx, s_STSTR_KFAADDED);
}

void M_CheatNoClip(CCore* cx) {
  dsda_AddMessage(cx, (plyr->cheats ^= CF_NOCLIP) & CF_NOCLIP ? s_STSTR_NCON : s_STSTR_NCOFF);
}

static void cheat_noclip(CCore* cx, void* v) {
  (void)v;

  if (demorecording) {
    dsda_QueueExCmdNoClip();
    return;
  }

  M_CheatNoClip(cx);
}

// 'behold?' power-up cheats (modified for infinite duration -- killough)
static void cheat_pw(CCore* cx, void* v)
{
    int pw = *(int*)v;

  if (pw == pw_allmap)
    dsda_TrackFeature(uf_automap);

  if (pw == pw_infrared)
    dsda_TrackFeature(uf_liteamp);

  if (plyr->powers[pw])
    plyr->powers[pw] = pw!=pw_strength && pw!=pw_allmap;  // killough
  else
    {
      P_GivePower(plyr, pw);
      if (pw != pw_strength)
        plyr->powers[pw] = -1;      // infinite duration -- killough
    }
  dsda_AddMessage(cx, s_STSTR_BEHOLDX);
}

// 'behold' power-up menu
static void cheat_behold(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, s_STSTR_BEHOLD);
}

// 'clev' change-level cheat
static void cheat_clev(CCore* cx, void* v)
{
    char* buf = v;
  int epsd, map;

  if (gamemode == commercial)
  {
    epsd = 1; //jff was 0, but espd is 1-based
    map = (buf[0] - '0') * 10 + buf[1] - '0';
  }
  else
  {
    epsd = buf[0] - '0';
    map = buf[1] - '0';
  }

  if (dsda_ResolveCLEV(cx, &epsd, &map))
  {
    dsda_AddMessage(cx, s_STSTR_CLEV);
    G_DeferedInitNew(cx, gameskill, epsd, map);
  }
}

// 'mypos' for player position
static void cheat_mypos(CCore* cx, void* v) {
    (void)v;
    dsda_ToggleConfig(cx, dsda_config_coordinate_display, false);
}

// cph - cheat to toggle frame rate/rendering stats display
static void cheat_rate(CCore* cx, void* v) {
    (void)v;
    dsda_ToggleRenderStats();
}

// compatibility cheat

static void cheat_comp(CCore* cx, void* v)
{
    char* buf = v;
  compatibility_level = (buf[0] - '0') * 10 + buf[1] - '0';

  if (compatibility_level < 0 ||
      compatibility_level >= MAX_COMPATIBILITY_LEVEL ||
      (compatibility_level > 17 && compatibility_level < 21))
  {
    doom_printf(cx, "Invalid complevel");
  }
  else
  {
    G_Compatibility(); // this is missing options checking
    doom_printf("%s", comp_lev_str[compatibility_level]);
  }
}

// variable friction cheat
static void cheat_friction(CCore* cx, void* v) {
    (void)v;

    dsda_AddMessage(
        cx, (variable_friction = !variable_friction) ? "Variable Friction enabled"
                                                    : "Variable Friction disabled"
    );
}

// Pusher cheat
// phares 3/10/98
static void cheat_pushers(CCore* cx, void* v)
{
    (void)v;
    dsda_AddMessage(cx, (allow_pushers = !allow_pushers) ? "Pushers enabled" : "Pushers disabled");
}

static void cheat_massacre(CCore* cx, void* v) // jff 2/01/98 kill all monsters
{
    (void)v;
  // jff 02/01/98 'em' cheat - kill all monsters
  // partially taken from Chi's .46 port
  //
  // killough 2/7/98: cleaned up code and changed to use dprintf;
  // fixed lost soul bug (LSs left behind when PEs are killed)

  int killcount=0;
  thinker_t *currentthinker = NULL;
  extern void A_PainDie(mobj_t *);

  // killough 7/20/98: kill friendly monsters only if no others to kill
  uint64_t mask = MF_FRIEND;
  P_MapStart();
  do
    while ((currentthinker = P_NextThinker(currentthinker,th_all)) != NULL)
    if (currentthinker->function == P_MobjThinker &&
  !(((mobj_t *) currentthinker)->flags & mask) && // killough 7/20/98
        (((mobj_t *) currentthinker)->flags & MF_COUNTKILL ||
         ((mobj_t *) currentthinker)->type == MT_SKULL))
      { // killough 3/6/98: kill even if PE is dead
        if (((mobj_t *) currentthinker)->health > 0)
          {
            killcount++;
            P_DamageMobj(cx, (mobj_t *)currentthinker, NULL, NULL, 10000);
          }
        if (((mobj_t *) currentthinker)->type == MT_PAIN)
          {
            A_PainDie((mobj_t *) currentthinker);    // killough 2/8/98
            P_SetMobjState (cx, (mobj_t *) currentthinker, S_PAIN_DIE6);
          }
      }
  while (!killcount && mask ? mask = 0, 1 : 0); // killough 7/20/98
  P_MapEnd();
  // killough 3/22/98: make more intelligent about plural
  // Ty 03/27/98 - string(s) *not* externalized
  doom_printf(cx, "%d Monster%s Killed", killcount, killcount==1 ? "" : "s");
}

void M_CheatIDDT(void)
{
  extern int dsda_reveal_map;

  dsda_TrackFeature(uf_iddt);

  dsda_reveal_map = (dsda_reveal_map+1) % 3;
}

// killough 2/7/98: move iddt cheat from am_map.c to here
// killough 3/26/98: emulate Doom better
static void cheat_ddt(CCore* cx, void* v) {
    (void)v;

    if (automap_input)
        M_CheatIDDT();
}

static void cheat_reveal_secret(CCore* cx, void* v)
{
  static int last_secret = -1;

  if (automap_input)
  {
    int i, start_i;

    dsda_TrackFeature(uf_iddt);

    i = last_secret + 1;
    if (i >= numsectors)
      i = 0;
    start_i = i;

    do
    {
      sector_t *sec = &sectors[i];

      if (P_IsSecret(sec))
      {
        dsda_UpdateIntConfig(cx, dsda_config_automap_follow, false, true);

        // This is probably not necessary
        if (sec->lines && sec->lines[0] && sec->lines[0]->v1)
        {
          AM_SetMapCenter(sec->lines[0]->v1->x, sec->lines[0]->v1->y);
          last_secret = i;
          break;
        }
      }

      i++;
      if (i >= numsectors)
        i = 0;
    } while (i != start_i);
  }
}

static void cheat_cycle_mobj(CCore* cx, mobj_t **last_mobj, int *last_count, int flags, int alive)
{
  extern int init_thinkers_count;
  thinker_t *th, *start_th;

  // If the thinkers have been wiped, addresses are invalid
  if (*last_count != init_thinkers_count)
  {
    *last_count = init_thinkers_count;
    *last_mobj = NULL;
  }

  if (*last_mobj)
    th = &(*last_mobj)->thinker;
  else
    th = &thinkercap;

  start_th = th;

  do
  {
    th = th->next;
    if (th->function == P_MobjThinker)
    {
      mobj_t *mobj;

      mobj = (mobj_t *) th;

      if ((!alive || mobj->health > 0) && mobj->flags & flags)
      {
        dsda_UpdateIntConfig(cx, dsda_config_automap_follow, false, true);
        AM_SetMapCenter(mobj->x, mobj->y);
        P_SetTarget(last_mobj, mobj);
        break;
      }
    }
  } while (th != start_th);
}

static void cheat_reveal_kill(CCore* cx, void* v)
{
  if (automap_input)
  {
    static int last_count;
    static mobj_t *last_mobj;

    dsda_TrackFeature(uf_iddt);

    cheat_cycle_mobj(cx, &last_mobj, &last_count, MF_COUNTKILL, true);
  }
}

static void cheat_reveal_item(CCore* cx, void* v)
{
    (void)v;

  if (automap_input)
  {
    static int last_count;
    static mobj_t *last_mobj;

    dsda_TrackFeature(uf_iddt);

    cheat_cycle_mobj(cx, &last_mobj, &last_count, MF_COUNTITEM, false);
  }
}

// killough 2/7/98: HOM autodetection
static void cheat_hom(CCore* cx, void* v) {
  (void)v;

  dsda_AddMessage(
	  cx, dsda_ToggleConfig(cx, dsda_config_flashing_hom, true) ? "HOM Detection On"
																: "HOM Detection Off"
  );
}

// killough 3/6/98: -fast parameter toggle
static void cheat_fast(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, (fastparm = !fastparm) ? "Fast Monsters On" : "Fast Monsters Off");
    dsda_RefreshGameSkill(); // refresh fast monsters
}

// killough 2/16/98: keycard/skullkey cheat functions
static void cheat_tntkey(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, "Red, Yellow, Blue");
}

static void cheat_tntkeyx(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, "Card, Skull");
}

static void cheat_tntkeyxx(CCore* cx, void* v) {
    int key = *(int*)v;
    dsda_AddMessage(cx, (plyr->cards[key] = !plyr->cards[key]) ? "Key Added" : "Key Removed");
}

// killough 2/16/98: generalized weapon cheats

static void cheat_tntweap(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, gamemode == commercial ? "Weapon number 1-9" : "Weapon number 1-8");
}

static void cheat_tntweapx(CCore* cx, void* v)
{
    char* buf = v;
  int w = *buf - '1';

  if ((w==wp_supershotgun && gamemode!=commercial) ||      // killough 2/28/98
      ((w==wp_bfg || w==wp_plasma) && gamemode==shareware))
    return;

  if (w == wp_fist) {
    // make '1' apply beserker strength toggle
    int p = pw_strength;
    cheat_pw(cx, &p);
  }
  else
    if (w >= 0 && w < NUMWEAPONS) {
      if ((plyr->weaponowned[w] = !plyr->weaponowned[w]))
        dsda_AddMessage(cx, "Weapon Added");
      else
        {
          dsda_AddMessage(cx, "Weapon Removed");
          if (w == plyr->readyweapon)         // maybe switch if weapon removed
            plyr->pendingweapon = P_SwitchWeapon(plyr);
        }
    }
}

// killough 2/16/98: generalized ammo cheats
static void cheat_tntammo(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, "Ammo 1-4, Backpack");
}

static void cheat_tntammox(CCore* cx, void* v)
{
    char* buf = v;

  int a = *buf - '1';
  if (*buf == 'b')  // Ty 03/27/98 - strings *not* externalized
    if ((plyr->backpack = !plyr->backpack))
    {
      dsda_AddMessage(cx, "Backpack Added");
      for (a = 0; a < NUMAMMO; a++)
        plyr->maxammo[a] <<= 1;
    }
    else
    {
      dsda_AddMessage(cx, "Backpack Removed");
      for (a = 0; a < NUMAMMO; a++)
        if (plyr->ammo[a] > (plyr->maxammo[a] >>= 1))
          plyr->ammo[a] = plyr->maxammo[a];
    }
  else
    if (a>=0 && a<NUMAMMO)  // Ty 03/27/98 - *not* externalized
      { // killough 5/5/98: switch plasma and rockets for now -- KLUDGE
        a = a==am_cell ? am_misl : a==am_misl ? am_cell : a;  // HACK
        dsda_AddMessage(cx, (plyr->ammo[a] = !plyr->ammo[a]) ?
                        plyr->ammo[a] = plyr->maxammo[a], "Ammo Added" : "Ammo Removed");
      }
}

static void cheat_smart(CCore* cx, void* v) {
    (void)v;
    dsda_AddMessage(cx, (monsters_remember = !monsters_remember) ?
                  "Smart Monsters Enabled" : "Smart Monsters Disabled");
}

static void cheat_pitch(CCore* cx, void* v) {
    (void)v;
  dsda_AddMessage(
    cx,
	  dsda_ToggleConfig(cx, dsda_config_pitched_sounds, true) ? "Pitch Effects Enabled"
															  : "Pitch Effects Disabled"
  );
}

static void cheat_notarget(CCore* cx, void* v)
{
    (void)v;

  plyr->cheats ^= CF_NOTARGET;
  if (plyr->cheats & CF_NOTARGET)
    dsda_AddMessage(cx, "Notarget Mode ON");
  else
    dsda_AddMessage(cx, "Notarget Mode OFF");
}

static void cheat_freeze(CCore* cx, void* v)
{
    (void)v;
  dsda_ToggleFrozenMode();
  dsda_AddMessage(cx, dsda_FrozenMode() ? "FREEZE ON" : "FREEZE OFF");
}

static void cheat_fly(CCore* cx, void* v)
{
    (void)v;

  if (plyr->mo != NULL)
  {
    if (raven)
    {
      if (plyr->powers[pw_flight])
      {
        P_PlayerEndFlight(plyr);
        plyr->powers[pw_flight] = 0;
        dsda_AddMessage(cx, "Fly mode OFF");
      }
      else
      {
        P_GivePower(plyr, pw_flight);
        plyr->powers[pw_flight] = INT_MAX;
        dsda_AddMessage(cx, "Fly mode ON");
      }
    }
    else
    {
      plyr->cheats ^= CF_FLY;
      if (plyr->cheats & CF_FLY)
      {
        plyr->mo->flags |= MF_NOGRAVITY;
        plyr->mo->flags |= MF_FLY;
        dsda_AddMessage(cx, "Fly mode ON");
      }
      else
      {
        plyr->mo->flags &= ~MF_NOGRAVITY;
        plyr->mo->flags &= ~MF_FLY;
        dsda_AddMessage(cx, "Fly mode OFF");
      }
    }
  }
}

static dboolean M_ClassicDemo(void)
{
  return (demorecording || demoplayback) && !dsda_AllowCasualExCmdFeatures();
}

static dboolean M_CheatAllowed(int when)
{
  return !dsda_StrictMode() &&
         !(when & not_demo         && (demorecording || demoplayback)) &&
         !(when & not_classic_demo && M_ClassicDemo()) &&
         !(when & not_menu         && menuactive);
}

static void cht_InitCheats(void)
{
  static int init = false;

  if (!init)
  {
    cheatseq_t* cht;

    init = true;

    for (cht = cheat; cht->cheat; cht++)
    {
      cht->sequence_len = strlen(cht->cheat);
    }
  }
}

//
// CHEAT SEQUENCE PACKAGE
//

//
// Called in st_stuff module, which handles the input.
// Returns a 1 if the cheat was successful, 0 if failed.
//
static int M_FindCheats(CCore* cx, int key)
{
  int rc = 0;
  cheatseq_t* cht;
  char char_key;

  cht_InitCheats();

  char_key = (char)key;

  for (cht = cheat; cht->cheat; cht++)
  {
    if (M_CheatAllowed(cht->when))
    {
      if (cht->chars_read < cht->sequence_len)
      {
        // still reading characters from the cheat code
        // and verifying.  reset back to the beginning
        // if a key is wrong

        if (char_key == cht->cheat[cht->chars_read])
          ++cht->chars_read;
        else if (char_key == cht->cheat[0])
          cht->chars_read = 1;
        else
          cht->chars_read = 0;

        cht->param_chars_read = 0;
      }
      else if (cht->param_chars_read < -cht->arg)
      {
        // we have passed the end of the cheat sequence and are
        // entering parameters now

        cht->parameter_buf[cht->param_chars_read] = char_key;

        ++cht->param_chars_read;

        // affirmative response
        rc = 1;
      }

      if (cht->chars_read >= cht->sequence_len &&
          cht->param_chars_read >= -cht->arg)
      {
        if (cht->param_chars_read)
        {
          static char argbuf[CHEAT_ARGS_MAX + 1];

          // process the arg buffer
          memcpy(argbuf, cht->parameter_buf, -cht->arg);

          cht->func(cx, argbuf);
        }
        else
        {
          // call cheat handler
          cht->func(cx, &cht->arg);

          if (cht->repeatable)
          {
            --cht->chars_read;
          }
        }

        if (!cht->repeatable)
          cht->chars_read = cht->param_chars_read = 0;
        rc = 1;
      }
    }
  }

  return rc;
}

typedef struct cheat_input_s {
  int input;
  const cheat_when_t when;
  void (*const func)();
  const int arg;
} cheat_input_t;

static cheat_input_t cheat_input[] = {
  { dsda_input_iddqd, not_classic_demo, cheat_god, 0 },
  { dsda_input_idkfa, not_demo, cheat_kfa, 0 },
  { dsda_input_idfa, not_demo, cheat_fa, 0 },
  { dsda_input_idclip, not_classic_demo, cheat_noclip, 0 },
  { dsda_input_idbeholdh, not_demo, cheat_health, 0 },
  { dsda_input_idbeholdm, not_demo, cheat_megaarmour, 0 },
  { dsda_input_idbeholdv, not_demo, cheat_pw, pw_invulnerability },
  { dsda_input_idbeholds, not_demo, cheat_pw, pw_strength },
  { dsda_input_idbeholdi, not_demo, cheat_pw, pw_invisibility },
  { dsda_input_idbeholdr, not_demo, cheat_pw, pw_ironfeet },
  { dsda_input_idbeholda, cht_always, cheat_pw, pw_allmap },
  { dsda_input_idbeholdl, cht_always, cheat_pw, pw_infrared },
  { dsda_input_idmypos, cht_always, cheat_mypos, 0 },
  { dsda_input_idrate, cht_always, cheat_rate, 0 },
  { dsda_input_iddt, cht_always, cheat_ddt, 0 },
  { dsda_input_ponce, not_demo, cheat_reset_health, 0 },
  { dsda_input_shazam, not_demo, cheat_tome, 0 },
  { dsda_input_chicken, not_demo, cheat_chicken, 0 },
  { dsda_input_notarget, not_demo, cheat_notarget, 0 },
  { dsda_input_freeze, not_demo, cheat_freeze, 0 },
  { 0 }
};

dboolean M_CheatResponder(CCore* cx, event_t *ev)
{
  cheat_input_t* cheat_i;

  if (dsda_ProcessCheatCodes() &&
      ev->type == ev_keydown &&
      M_FindCheats(cx, ev->data1.i))
    return true;

  for (cheat_i = cheat_input; cheat_i->input; cheat_i++)
  {
    if (dsda_InputActivated(cheat_i->input))
    {
      if (M_CheatAllowed(cheat_i->when))
        cheat_i->func(cheat_i->arg);

      return true;
    }
  }

  if (M_CheatAllowed(not_demo) && dsda_InputActivated(dsda_input_avj))
  {
    plyr->mo->momz = 1000 * FRACUNIT / plyr->mo->info->mass;

    return true;
  }

  return false;
}

dboolean M_CheatEntered(CCore* cx, const char* element, const char* value)
{
  cheatseq_t* cheat_i;

  for (cheat_i = cheat; cheat_i->cheat; cheat_i++)
  {
    if (!strcmp(cheat_i->cheat, element) && M_CheatAllowed(cheat_i->when & ~not_menu))
    {
      if (cheat_i->arg >= 0)
        cheat_i->func(cx, &cheat_i->arg);
      else
        cheat_i->func(cx, value);
      return true;
    }
  }
  return false;
}

// heretic

#include "p_user.h"

static void cheat_reset_health(CCore* cx, void* v)
{
    (void)v;

  if (heretic && plyr->chickenTics)
  {
    plyr->health = plyr->mo->health = MAXCHICKENHEALTH;
  }
  else if (hexen && plyr->morphTics)
  {
    plyr->health = plyr->mo->health = MAXMORPHHEALTH;
  }
  else
  {
    plyr->health = plyr->mo->health = MAXHEALTH;
  }
  dsda_AddMessage(cx, "FULL HEALTH");
}

static void cheat_artifact(CCore* cx, void* v)
{
    char* buf = v;

  int i;
  int j;
  int type;
  int count;

  if (!heretic) return;

  type = buf[0] - 'a' + 1;
  count = buf[1] - '0';
  if (type == 26 && count == 0)
  { // All artifacts
    for (i = arti_none + 1; i < NUMARTIFACTS; i++)
    {
      if (gamemode == shareware && (i == arti_superhealth || i == arti_teleport))
      {
        continue;
      }
      for (j = 0; j < 16; j++)
      {
        P_GiveArtifact(plyr, i, NULL);
      }
    }
    dsda_AddMessage(cx, "YOU GOT IT");
  }
  else if (type > arti_none && type < NUMARTIFACTS && count > 0 && count < 10)
  {
    if (gamemode == shareware && (type == arti_superhealth || type == arti_teleport))
    {
      dsda_AddMessage(cx, "BAD INPUT");
      return;
    }
    for (i = 0; i < count; i++)
    {
      P_GiveArtifact(plyr, type, NULL);
    }
    dsda_AddMessage(cx, "YOU GOT IT");
  }
  else
  {
    dsda_AddMessage(cx, "BAD INPUT");
  }
}

static void cheat_tome(CCore* cx, void* v)
{
    (void)v;

  if (!heretic) return;

  if (plyr->powers[pw_weaponlevel2])
  {
    plyr->powers[pw_weaponlevel2] = 0;
    dsda_AddMessage(cx, "POWER OFF");
  }
  else
  {
    P_UseArtifact(cx, plyr, arti_tomeofpower);
    dsda_AddMessage(cx, "POWER ON");
  }
}

static void cheat_chicken(CCore* cx, void* v) {
    (void)v;

  if (!raven) return;

  P_MapStart();
  if (heretic)
  {
    if (plyr->chickenTics)
    {
      if (P_UndoPlayerChicken(cx, plyr))
      {
          dsda_AddMessage(cx, "CHICKEN OFF");
      }
    }
    else if (P_ChickenMorphPlayer(cx, plyr))
    {
      dsda_AddMessage(cx, "CHICKEN ON");
    }
  }
  else
  {
    if (plyr->morphTics)
    {
      P_UndoPlayerMorph(cx, plyr);
    }
    else
    {
      P_MorphPlayer(cx, plyr);
    }
    dsda_AddMessage(cx, "SQUEAL!!");
  }
  P_MapEnd();
}

// hexen

#include "hexen/p_acs.h"

static void cheat_init(CCore* cx, void* v) {
    (void)v;

    if (dsda_ResolveINIT(cx)) {
        P_SetMessage(cx, plyr, "LEVEL WARP", true);
    }
}

static void cheat_inventory(CCore* cx, void* v)
{
    (void)v;

  int i, j;
  int start, end;

  if (!raven) return;

  if (heretic)
  {
    start = arti_none + 1;
    end = NUMARTIFACTS;
  }
  else
  {
    start = hexen_arti_none + 1;
    end = hexen_arti_firstpuzzitem;
  }

  for (i = start; i < end; i++)
  {
    for (j = 0; j < g_arti_limit; j++)
    {
      P_GiveArtifact(plyr, i, NULL);
    }
  }
  P_SetMessage(cx, plyr, "ALL ARTIFACTS", true);
}

static void cheat_puzzle(CCore* cx, void* v)
{
    (void)v;

  int i;

  if (!hexen) return;

  for (i = hexen_arti_firstpuzzitem; i < HEXEN_NUMARTIFACTS; i++)
  {
    P_GiveArtifact(plyr, i, NULL);
  }
  P_SetMessage(cx, plyr, "ALL PUZZLE ITEMS", true);
}

static void cheat_class(CCore* cx, void* v)
{
    char* buf = v;

  int i;
  int new_class;

  if (!hexen) return;

  if (plyr->morphTics)
  {                           // don't change class if the player is morphed
      return;
  }

  new_class = 1 + (buf[0] - '0');
  if (new_class > PCLASS_MAGE || new_class < PCLASS_FIGHTER)
  {
    P_SetMessage(cx, plyr, "INVALID PLAYER CLASS", true);
    return;
  }
  plyr->pclass = new_class;
  for (i = 0; i < NUMARMOR; i++)
  {
    plyr->armorpoints[i] = 0;
  }
  PlayerClass[consoleplayer] = new_class;
  P_PostMorphWeapon(cx, plyr, wp_first);
  SB_SetClassData();
  SB_Start();
  P_SetMessage(cx, plyr, "CLASS CHANGED", true);
}

static void cheat_script(CCore* cx, void* v)
{
    char* buf = v;

  int script;
  byte script_args[3];
  int tens, ones;
  static char textBuffer[40];

  if (!map_format.acs) return;

  tens = buf[0] - '0';
  ones = buf[1] - '0';
  script = tens * 10 + ones;
  if (script < 1)
      return;
  if (script > 99)
      return;
  script_args[0] = script_args[1] = script_args[2] = 0;

  if (P_StartACS(cx, script, 0, script_args, plyr->mo, NULL, 0))
  {
    snprintf(textBuffer, sizeof(textBuffer), "RUNNING SCRIPT %.2d", script);
    P_SetMessage(cx, plyr, textBuffer, true);
  }
}
