# Music sourcing for kaazap — research draft

Draft, 2026-09-24. Every web claim below was read on **2026-09-24** unless a
different date is given. "Unverified" means the primary page could not be read
(JavaScript-only, 403, 404) or disagreed with another source; the claim is
reported with its secondary source, not taken as settled. Nothing here was
listened to, so no track has passed the person's ear yet.

---

## 1. Scoring criteria (fixed)

1. **Ear** — does the person like it (only the person can score this).
2. **License** — clean for a free public itch.io release *and* a public portfolio.
3. **Chiptune fit** — swing / jazz / cantina feel on chip sounds, without copying anything.
4. **Cost of track 4 and track 8** — what the next track costs, not only the first.
5. **Repo size** — files are embedded in the binary (repo ~12 MB; the placeholder MP3 is 7.5 MB).
   For scale: a 45 s loop is about 0.7 MB as 128 kbps MP3, about 7.9 MB as 44.1 kHz 16-bit stereo WAV,
   about 2 MB as 22.05 kHz 16-bit mono WAV (arithmetic, not a source).
6. **Reproducibility** — a score or script in the repo beats a binary blob.
7. **The person's time.**

---

## 2. AI music services

Common background first, because it applies to all of them.

**Copyright status of the output (US).** The Copyright Office's *Copyright and AI, Part 2:
Copyrightability* (29 Jan 2025) concludes that AI output is protectable "only where a human author
has determined sufficient expressive elements", and that "the mere provision of prompts" is not
enough ([copyright.gov/newsnet/2025/1060](https://www.copyright.gov/newsnet/2025/1060.html);
report index at [copyright.gov/ai](https://copyright.gov/ai/)). The Supreme Court denied
certiorari in *Thaler v. Perlmutter* on 2 Mar 2026, leaving the human-authorship rule in place
([Mayer Brown](https://www.mayerbrown.com/en/insights/publications/2026/03/supreme-court-denies-review-in-ai-authorship-case),
secondary). **What it means for kaazap:** a prompted track is very likely not copyrightable by
anyone, so "you own it" in a service's terms is mostly a *contractual permission* plus a promise the
service won't claim it. kaazap only needs permission to use, not ownership, so that is enough for
use — but the person could not stop someone else copying the track, and the service's permission is
only as good as the service's own right to the training data.

**itch.io labelling.** itch.io's quality guidelines say creators must "accurately tag your project if
it contains materials produced by generative AI"
([itch.io/docs/creators/quality-guidelines](https://itch.io/docs/creators/quality-guidelines)). The
Nov 2024 staff post says the disclosure field is mandatory for asset pages and "encourage[d]" for
games ([itch.io/t/4309690](https://itch.io/t/4309690/generative-ai-disclosure-tagging)). The two pages
read slightly differently; in practice, **any AI track in kaazap means tagging the game "AI Generated
Sounds"** on itch.io and saying so in the portfolio.

### Per service

| Service | Who owns output / on which tier | Free public game + portfolio? | Attribution / label | Style control, loops, stems, length | Price of the tier that grants the rights | Legal status |
|---|---|---|---|---|---|---|
| **Suno** | Free: Suno owns, "lawful, personal and non-commercial" only. Pro/Premier: Suno "assigns to you all of its right, title and interest" ([suno.com/terms](https://suno.com/terms), updated 10 Aug 2026, effective 3 Sep 2026; [help.suno.com/en/articles/2746945](https://help.suno.com/en/articles/2746945)) | Paid tier only, and only for a "permitted download" (terms above). Free tier has no downloads ([suno.com/pricing](https://suno.com/pricing)) | None required; Suno may say publicly the output was made with Suno | Genre prompts; full songs; stems via "Get Stems" on Pro/Premier ([help.suno.com/en/articles/13925185](https://help.suno.com/en/articles/13925185), via search snippet — unverified). No seamless-loop mode found | Pro **$8/mo** (20 downloads/mo), Premier $24/mo (60) per [suno.com/pricing](https://suno.com/pricing). Several third-party pages say $10/$30 — **unverified which is current** | WMG settled + licensed (Nov 2025); BMG (Aug 2026); Believe (8 Sep 2026). **UMG and Sony sued again on 19 Sep 2026 over the new v6 model**; GEMA won in Munich (31 Jul 2026); KODA, SOCAN, Round Hill and an artist class action pending ([Wikipedia: Suno](https://en.wikipedia.org/wiki/Suno_(platform)); [Digital Music News 18 Sep 2026](https://www.digitalmusicnews.com/2026/09/18/universal-music-suno-lawsuit-second/), headline only) |
| **Udio** | Help page (updated 26 Mar 2025, pre-settlement) says Udio claims no ownership ([help.udio.com](https://help.udio.com/en/articles/10739216-answers-to-common-usage-questions)) | **No — downloads disabled since 30 Oct 2025** as part of the UMG settlement; still off as of Sep 2026 with no restore date ([Wikipedia: Udio](https://en.wikipedia.org/wiki/Udio); secondary reporting). Terms page did not render — **unverified** | n/a | n/a | n/a | Settled with UMG (Oct 2025) and reportedly WMG, Merlin, Kobalt; "walled garden" platform. **Out of the running.** |
| **Stable Audio (hosted)** | Paid tiers get a commercial "Creator" licence; free is non-commercial (secondary: [amrytt review](https://amrytt.com/stable-audio-review/)). stableaudio.com terms/pricing/FAQ pages render no text to a fetch — **unverified** | Paid only (unverified) | Unknown | Instrumental only; hosted app on 2.5, ~3 min cap (secondary) | Pro ~$11.99/mo (secondary, **unverified**) | Trained on licensed data (Stability's claim); no lawsuit found |
| **Stable Audio Open 1.0 / 3.0 weights** | "You own any outputs … to the extent permitted by applicable law" ([stability.ai/community-license-agreement](https://stability.ai/community-license-agreement), updated 5 Jul 2024) | Yes, free under $1M annual revenue | Licence asks for "Powered by Stability AI" and a notice file when you *distribute the model or derivative works*; whether that reaches a game that only ships the audio is **unclear** | Open 1.0: up to 47 s, 44.1 kHz, trained on Freesound + FMA CC audio, weak on some music styles ([HF model card](https://huggingface.co/stabilityai/stable-audio-open-1.0)). 3.0 Small (≤2 min) / Medium (≤6:20) open weights, inpainting and continuation ([Stability post](https://stability.ai/news-updates/meet-stable-audio-3-the-model-family-built-for-artistic-experimentation-with-open-weight-models); release date read as May 2025 on the page vs 20 May 2026 in coverage — **unresolved**) | $0 + local GPU time. Apple Silicon support not verified | Licensed training data (claim) |
| **ElevenLabs Music** | Terms don't state ownership explicitly; warn output "may not be unique" ([elevenlabs.io/music-terms](https://elevenlabs.io/music-terms), updated 26 May 2026) | Paid plans: "All paid plans include a commercial license". Free: no commercial use ([help article](https://elevenlabs.io/docs/help-center/legal/can-i-publish-the-content-i-generate-on-the-platform)) | Free tier must credit "Eleven Music"; paid none. Prompts may not name artists or song titles (music terms) | 3 s–5 min, instrumental option, section-by-section editing; MP3/WAV; no loop or stem export mentioned ([docs](https://elevenlabs.io/docs/overview/capabilities/music)) | **Starter $6/mo** lists "Music commercial use" ([elevenlabs.io/pricing](https://elevenlabs.io/pricing)) | Launched Aug 2025 with Merlin and Kobalt licences; claims licensed training data ([TechCrunch](https://techcrunch.com/2025/08/05/elevenlabs-launches-an-ai-music-generator-which-it-claims-is-cleared-for-commercial-use/), headline) |
| **Mubert (Render)** | "Mubert owns all the rights to the tracks generated"; users get a licence ([mubert.com/render/license](https://mubert.com/render/license)) | License page lists "Apps & games" for all tiers, commercial from paid tiers. Free (Ambassador) is non-commercial with attribution; its licence text reportedly limits it to personal social-network use — **conflicting, unverified** | Free/Creator: attribution required; Pro+: none ([pricing](https://mubert.com/render/pricing)) | Genre/mood driven, up to 25 min on paid; WAV on paid. Loop support **unverified**. No Content ID, no streaming release | Pricing page shows Creator $14/mo marked non-commercial and Pro $39/mo as the first "Commercial use" tier ([pricing](https://mubert.com/render/pricing)); licence page says commercial from Creator — **conflicting** | No lawsuit found |
| **AIVA** | Free and Standard: "Copyright owned by AIVA". Pro: "Copyright owned by YOU" ([aiva.ai/pricing](https://www.aiva.ai/pricing)) | Free: non-commercial, "ANY use cases that are meant to be non-for-profit", with credit ([AIVA helpdesk](https://aiva.crisp.help/en/article/i-dont-understand-the-terms-of-license-1wqvh5v/)). Games not named. Pro: "ANY way" | Free: credit line "Soundtrack composed by AIVA …" | Style presets and editable arrangement; **exports MIDI on every tier** (3 downloads/mo free); ≤3 min free | Pro €33/mo billed annually | No lawsuit found. Symbolic (MIDI) model |
| **Soundraw** | "You own the generated music upon download" ([soundraw.io/license](https://soundraw.io/license)) | Yes: "video games" named; perpetual after cancelling (except lo-fi/meditation as-is) | Optional | Preset genres, length control, bar-level editing; stems on Artist Pro+ ([pricing](https://soundraw.io/pricing)). Whether a jazz or chiptune preset exists **not verified** | Creator $16.99/mo ($11.04/mo annual); no free downloads | Trained on in-house music (claim) |
| **Beatoven.ai** | "Beatoven owns the copyright"; perpetual, non-exclusive licence covering games ([beatoven.ai/tos](https://www.beatoven.ai/tos), updated 5 Jun 2024). Wording "perpetual … during the term of this Agreement" is self-contradictory | Yes (paid download) | Credit "Music by Beatoven.ai" where practicable (ToS) | Mood/genre, stems | Pricing page 404; third-party figures ($2.50–$16.66/mo) **unverified** | Fairly Trained certified (site claim) |
| **Google Lyria / MusicFX** | Gemini API terms: "Google won't claim ownership over that content" ([ai.google.dev/gemini-api/terms](https://ai.google.dev/gemini-api/terms), effective 23 Mar 2026) | API: yes for paid use (free tier data is used for training). Gemini app / Flow Music free-vs-paid commercial rules: third-party only — **unverified** | Every output carries a SynthID watermark ([Google blog, 25 Mar 2026](https://blog.google/innovation-and-ai/technology/ai/lyria-3-pro/)) | Lyria 3 Pro: up to 3 min, structure control; instrumental option in Gemini app ([gemini.google](https://gemini.google/overview/music-generation/)). **MusicFX and MusicFX DJ shut down 31 Jul 2026**, replaced by Google Flow Music ([AlphaSignal](https://alphasignal.ai/news/google-kills-musicfx-to-go-all-in-on-flow-music), secondary) | Gemini API pay-as-you-go (Lyria price not checked); Flow Music tiers **unverified** | Google says licensed data (secondary) |
| **Meta MusicGen (weights)** | Weights **CC-BY-NC 4.0**, code MIT; intended use is research ([HF model card](https://huggingface.co/facebook/musicgen-large)) | **Weak**: the non-commercial weight licence makes a public portfolio use arguable at best; output status not addressed | n/a | Instrumental only; trained on Meta/Shutterstock/Pond5 licensed music; needs a GPU with ~16 GB for the medium model ([AudioCraft docs](https://github.com/facebookresearch/audiocraft/blob/main/docs/MUSICGEN.md)) | $0 + GPU | Licensed training data |

**Provenance risk, honestly.** None of these services can show that a given output quotes nothing.
Suno's newest model is being sued on the theory that training on the old model's outputs "launders"
the old infringement; Udio stopped letting anyone take audio out at all; the "licensed data" services
(ElevenLabs, Stable Audio, Soundraw, Beatoven, Google) are stating a training-data policy, not
certifying each output. For a *cantina* brief specifically, the risk is sharper than usual: the most
famous piece in that exact style is heavily represented in any web-scale corpus, and a prompt that
says "cantina band" aims straight at it (ElevenLabs forbids naming songs or artists in prompts for
this reason). A portfolio piece that is later found to echo a famous melody is the wrong kind of
memorable, and the person could not prove otherwise. Add the itch.io tag and the fact that the result
isn't copyrightable, and AI output is the weakest fit for "a portfolio piece that shows my work".

---

## 3. Commissioning a human chiptune composer

**Rates seen (all read 2026-09-24):**

| Source | Rate | Note |
|---|---|---|
| Fiverr gig search results (Zakdcomposer, Prodnematoda, Msoundworks, Beatscribe, Alex_bit) | $15–$45 starting packages; e.g. "60-second musical loop … $15", "30-second loop-able … $20, 2-day delivery", "$25 within 5 days" | Search snippets only; Fiverr returned 403 so package contents and whether commercial rights cost extra are **unverified** ([Fiverr search](https://www.fiverr.com/search/gigs?query=chiptune)) |
| itch.io Help Wanted, Taiel Ramírez (4 Jul 2025) | $15 per 30 s track, $30 per finished minute, loop-ready versions | [itch.io/post/13283575](https://itch.io/post/13283575/view-in-topic) |
| itch.io, chiptune composer (22 Jan 2026) | "$15–$20 an hour" | [itch.io/t/5811648](https://itch.io/t/5811648/paid-chiptune-composer-looking-to-join-a-serious-project) |
| itch.io, composer + audio programmer (30 Nov 2025) | $200 per finished minute, "open to lower rates" | [itch.io/t/5596427](https://itch.io/t/5596427/music-composer-audio-programmer-paid-200minute-flexible-revshare-welcome) |
| Twine rate guide (17 Sep 2025) | Entry $50–150/min, mid $150–300/min, veteran $300–1000+/min; simple electronic $50–200/min | [twine.net](https://www.twine.net/blog/game-composer-pricing/) |
| Ninichi (6 Nov 2018) | $50–$2500/min overall; experienced indie $200–1000/min | [ninichimusic.com](https://ninichimusic.com/blog/understanding-how-much-an-indie-game-music-composer-costs) |

**Reading:** for a 30–60 s loop, the realistic band is **~$15–$50 per track** from a hobby/entry
composer on Fiverr or itch.io, **~$100–$300** from a working indie composer. A fourth track costs the
same as the first; an eighth may come cheaper if the same composer does the set and reuses the
instruments. r/gamedev and r/chiptunes threads were not read (no Reddit pages were fetched), so no
figures come from there.

**Where to find them:** itch.io's "Help Wanted or Offered" board (where most of the above came from),
Fiverr's chiptune gig category, OpenGameArt authors whose CC work fits (they can be approached for
paid custom work — several list a contact), and r/gameDevClassifieds (not read).

**Turnaround:** the only concrete figures are Fiverr snippets (2 and 5 days for a short loop,
unverified). Plan on one to two weeks per track including a round of revisions; that is an estimate,
not a sourced figure.

**What a simple agreement should grant** (drawn from [GameSoundCon on work-for-hire](https://www.gamesoundcon.com/post/2016/07/17/composing-music-for-games-and-work-for-hire-deal-or-no-deal), updated 8 Apr 2020, and the Twine guide above):

- The composer warrants the work is **original** — no quotation of existing melodies, no samples they
  lack rights to, and **no AI generation** unless disclosed.
- A **perpetual, worldwide, irrevocable licence** to use, reproduce and distribute the tracks in the
  game (free or paid, any platform) **and in promotional material** (trailers, store page, portfolio,
  devlog videos).
- **Non-exclusive** is enough and is cheaper; the composer keeps copyright and can sell the soundtrack.
  Exclusive or work-for-hire costs more and buys control nobody here needs.
- **Derivatives allowed**: the right to edit, loop, re-arrange and make in-house regional variations.
  Ask for the **project file** (FamiTracker/FamiStudio/Furnace/MIDI) as a deliverable — it makes
  variations possible and puts a score in the repo.
- **Credit**: the composer is named in the game credits and README; the composer may show the work in
  their own portfolio.

---

## 4. Library survey

Search terms used on each site: chiptune + jazz / swing / bebop / big band / cantina. Nothing below
has been listened to.

| Title | Author | Licence | Link | Length / size | Style note | Derivatives? |
|---|---|---|---|---|---|---|
| NES chiptune "Swingshot" (Swing Jazz) | Haley Halcyon | **CC0** | [OGA](https://opengameart.org/content/nes-chiptune-swingshot-swing-jazz) | length not listed; looping OGG 1.3 MB, loop point 11.36 s; **FamiTracker project (.ftm, 1.4 KB) included** | 12-bar blues in B♭, plain 2A03, swing | Yes |
| NES chiptune "Slam-Funk" | Haley Halcyon | CC-BY 4.0 | [OGA](https://opengameart.org/content/nes-chiptune-slam-funk) | looping OGG 1.5 MB, loop 15.74 s; source included | funk/jazz NES | Yes, with credit |
| Talking Cute (Chiptune) | Pro Sensory (Alex McCulloch) | **CC0** (credit requested) | [OGA](https://opengameart.org/content/talking-cute-chiptune) | MP3 3.6 MB | jazz-influenced chip; 1,700+ downloads | Yes |
| Bebop (Chiptune) | Pro Sensory | CC0 | [OGA](https://opengameart.org/content/bebop-chiptune) | MP3 ~4.1 MB | jazz/pop chip | Yes |
| Action Packed Jazz Chiptune | Pro Sensory | CC0 | [OGA](https://opengameart.org/content/action-packed-jazz-chiptune) | MP3 2.2 MB / OGG 1.3 MB | jazz chip, Ionian, action | Yes |
| Aug 1 Chiptunes (two "jazz_chiptune" files) | Pro Sensory | CC0 | [OGA](https://opengameart.org/content/aug-1-chiptunes) | MP3 2.8 MB each | jazz chip | Yes |
| N163 Swing Rhythm | Spring Spring | CC0 | [OGA](https://opengameart.org/content/n163-swing-rhythm) | OGG 883 KB ("short") | swing on Namco N163 | Yes |
| Beat 'Em Up Jazz Club | Spring Spring | CC0 | [OGA](https://opengameart.org/content/beat-em-up-jazz-club) | OGG 1.3 MB ("short") | jazzy, Mega Drive FM | Yes |
| Nintendo Style – Quirky Music Loop 01 | PlayOnLoop | CC-BY 4.0 | [OGA](https://opengameart.org/content/nintendo-style-quirky-music-loop-01) | ZIP 2.2 MB, loops | "jazzy melody with a swing groove" | Yes, with credit |
| Free NES Music Pack (incl. "nes_07-jazz") | pmiller | CC0 | [OGA](https://opengameart.org/content/free-nes-music-pack) | 11 tracks, 51.7 MB zip; FamiTracker sources included | one jazz, one blues track | Yes |
| 8-bit Pack 2 ("Red Heels" etc.) | TAD | CC-BY 4.0 | [OGA](https://opengameart.org/content/8-bit-pack-2) | 7 tracks, 2.1–3.6 MB MP3 | tagged jazz/hiphop; made in GarageBand, so chip fit uncertain | Yes, with credit |
| A Jazz-bit Story 2 (12 tracks) | AvapXia | **CC BY** | [FMA](https://freemusicarchive.org/music/avapxia/a-jazz-bit-story-2) | 2:22–5:05 each (e.g. "Time and Soul" 2:22, "Luna's Run" 2:30, "Boogie!" 3:55) | "jazz songs written using chiptune instruments" (album, Nov 2021) | Yes, with credit |
| multiverse (5 tracks) | miha mōyo | CC BY-**ND** | [FMA](https://freemusicarchive.org/music/miha-moyo/multiverse) | 1:54–3:30 | jazz / pop / chiptune | **No** — trimming to a loop is arguably an adaptation |
| Big Band Swing with a Chiptune Twist | NickPanek | Pixabay Content License | [Pixabay](https://pixabay.com/music/traditional-jazz-big-band-swing-with-a-chiptune-twist-332017/) | 1:05 | closest match on title, but **the page marks it "AI Generated: Yes"** | Yes (licence allows modification; no standalone redistribution — [summary](https://pixabay.com/service/license-summary/)) |
| Quinklette | Cakeflaps | CC-BY 4.0 / OGA-BY 3.0 / CC0 | [OGA](https://opengameart.org/content/quinklette) | WAV 29.9 MB / OGG 1.7 MB, loops | tagged "cantina" but **not described as chiptune** | Yes |

**Sites with nothing usable found:** Freesound (a "chiptune swing" search gave four short loops/SFX;
"chiptune jazz" gave none); incompetech (no chiptune genre filter; Kevin MacLeod's 8-bit titles are
not described as swing, and the current placeholder already comes from here); itch.io asset packs
(the chiptune+jazz tag page lists five packs, none described as swing/jazz chip; "Electro Swing" packs
exist but aren't chiptune); ccMixter (search page returned no readable results); **Newgrounds**
(audio search and terms pages returned 403 — **not surveyed**).

**Does anything fit?** Several tracks are *plausible* on paper; none is described as "cantina" and
chiptune at once, and none has been heard.

**A matching set of four from one author?** Two candidates:
- **Pro Sensory / Alex McCulloch, CC0** — five jazz-chip tracks (Talking Cute, Bebop, Action Packed
  Jazz, two Aug 1 jazz tracks). Cleanest licence possible; the risk is that they don't sound like a set
  or like a cantina.
- **AvapXia, CC BY** — twelve jazz-chip tracks from one album, so they will share a palette. They are
  2–5 min songs, not loops: each would need a loop cut (permitted under CC BY) and the files would
  weigh more.

Haley Halcyon's "Swingshot" is the best single fit on paper and the only one that ships its **score**
(a FamiTracker file) — but that author has only one swing piece, so it can't anchor a set.

---

## 5. Tooling: score in the repo, rendered by a real chip engine

| Tool | macOS | Command-line render | Text / import format | Licence |
|---|---|---|---|---|
| **FamiStudio** (NES 2A03 + expansion chips) | macOS 10.15+, Intel and Apple Silicon; needs the **.NET 8 runtime**; unsigned, so Gatekeeper must be overridden once ([install docs](https://famistudio.org/doc/install/)). Not in Homebrew (`brew search`, 2026-09-24) | Yes: `FamiStudio music.fms wav-export music.wav -export-songs:0 -wav-export-rate:48000`; "import from any supported format and export to any supported format via the command line" ([cmdline docs](https://famistudio.org/doc/cmdline/)). The exact executable path inside the macOS `.app` is **not documented** on the pages read | Imports **FamiStudio Text** (native, hierarchical `Project > Instrument/Song > Channel > Pattern > Note`, `Attribute="Value"`; "not forward or backward compatible" across versions), FamiTracker TXT/FTM, MIDI, NSF, VGM ([import](https://famistudio.org/doc/import/), [export](https://famistudio.org/doc/export/)). WAV export can play N times or a set duration, and write a separate intro file | MIT ([GitHub](https://github.com/BleuBleu/FamiStudio)) |
| **Furnace** (dozens of chips: NES, Game Boy, Genesis FM, SID …) | macOS incl. Apple Silicon ([GitHub](https://github.com/tildearrow/furnace)). **Not in Homebrew** despite a third-party claim (`brew search furnace` found nothing on 2026-09-24; README's package list names Linux/BSD/Nix only) | Yes: `-output <file>` "output audio to file", `-loops`, `-subsong`, `-outmode` (per chip/channel), `-console` ([src/main.cpp](https://raw.githubusercontent.com/tildearrow/furnace/master/src/main.cpp)) | Native `.fur` is a documented **binary** (optionally zlib) format ([format.md](https://github.com/tildearrow/furnace/blob/master/papers/format.md)); `-txtout` exports text but no text *import* was found | GPL-2.0-or-later |
| **FluidSynth + chiptune SoundFont** | `brew install fluid-synth` — 2.6.1, bottled, LGPL-2.1-or-later (`brew info`, 2026-09-24) | `fluidsynth -ni soundfont.sf2 song.mid` plus `-F out.wav -r 44100` for fast render to file ([user manual](https://www.fluidsynth.org/wiki/UserManual)) | Standard MIDI | LGPL-2.1+ |
| **Python MIDI writing** | Python 3.11 already present | n/a (writes the `.mid`) | The stdlib has **no MIDI module**; hand-writing SMF bytes with `struct` is ~100 lines. `mido` 1.3.3, MIT, no required dependencies for file I/O ([PyPI](https://pypi.org/project/mido/)) | — |
| **Python stdlib synthesis** (not in the brief, but relevant) | as above | `wave` writes uncompressed PCM WAV ([docs](https://docs.python.org/3/library/wave.html)) | A script generates square/triangle/noise samples directly | — |

**Chiptune SoundFonts:** *Jummbox SoundFont* by stgiga, GM-mapped chip/FM, **CC-BY-SA 4.0**, but
about **1 GB** ([itch.io](https://stgiga.itch.io/jummboxsoundfont)) — fine to download at render time,
not to commit; whether ShareAlike reaches audio rendered with it is an open question. *8bitSF* is
listed on Musical Artifacts as licence unknown / non-free (samples from Nintendo games), so **avoid
it** (search snippet; the page returned 403). A "NES 8-Bit Soundfont" under CC-BY 3.0 appears in
search results but its page returned 403 — **unverified**. A licence-clean chip SoundFont was not
confirmed; the stdlib synthesis route sidesteps the question.

**What each pipeline would look like on macOS**

1. **FamiStudio text → WAV.** Commit `music/*.txt` (FamiStudio Text) — or a Python script that writes
   it — plus a `render.sh` that calls the FamiStudio CLI with `wav-export`. Install: .NET 8 runtime +
   the FamiStudio zip, one Gatekeeper override. Authentic 2A03 sound; swing is done by alternating
   row lengths (groove), which trackers support. Risk: the text format is version-locked, so pin the
   FamiStudio version in the README. Swingshot's `.ftm` can be imported here as a starting point.
2. **Furnace `.fur` → WAV.** Compose in the GUI, commit the `.fur` (small but binary) and a render
   script using `-output`. Install: the Furnace app from GitHub releases. Richest chip palette
   (Genesis FM brass would suit big-band), but the score isn't diffable text.
3. **Python → MIDI → FluidSynth → WAV.** Commit a Python score (notes as data) that writes `.mid`
   with `mido` or hand-rolled bytes, then `fluidsynth -ni -F out.wav font.sf2 song.mid`. Install:
   `brew install fluid-synth`, optionally `pip install mido`, plus a SoundFont. Fully text,
   diffable; the sound depends on finding a licence-clean chip SoundFont.
4. **Python stdlib only → WAV.** One script holds the score and synthesises pulse/triangle/noise
   voices with `wave` + `array`. Nothing to install; fully reproducible; the look-and-feel is
   whatever the script's synth can do, and the person's time goes into writing the synth.

All four can render 22.05 kHz mono WAV or be converted to MP3 to keep the embedded size near 1 MB
per loop.

---

## 6. Summary

| Option | Ear | License | Chiptune fit | Track 4 / track 8 cost | Repo size | Reproducibility | Person's time |
|---|---|---|---|---|---|---|---|
| Suno Pro | untested; strong swing | contractual OK; label suits live; AI tag | prompts can; loops not native | ~$8/mo, 20 dl/mo | MP3 ~1 MB/loop | blob | low per try, high to curate |
| Udio | — | no downloads | — | — | — | — | **dead** |
| Stable Audio hosted | untested | paid tier (unverified) | weak on style (2.5) | ~$12/mo (unverified) | ~1 MB | blob | low |
| Stable Audio Open / 3.0 local | untested | Community Licence, free <$1M; notice ambiguity | weak on music (1.0) | $0 + GPU | ~1 MB | model + seed + prompt, semi | high (local setup) |
| ElevenLabs Music | untested | paid Starter OK; no artist/title prompts | prompt-driven | $6/mo | ~1 MB | blob | low |
| Mubert | untested | owner = Mubert; tier rules conflict | genre presets | $14–$39/mo | ~1 MB | blob | low |
| AIVA | untested | free = non-profit + credit (grey for portfolio); Pro €33/mo owns | MIDI export → feed pipelines 1/3/4 | free 3/mo; Pro €33/mo | tiny if MIDI | MIDI in repo | medium |
| Soundraw | untested | clean, games named | presets; jazz/chip unverified | $16.99/mo | ~1 MB | blob | low |
| Beatoven | untested | Beatoven owns; credit | presets | unverified | ~1 MB | blob | low |
| Google Lyria / Flow | untested | API: Google claims no ownership; SynthID | prompt-driven | unverified | ~1 MB | blob | low |
| MusicGen weights | untested | CC-BY-NC weights | fair | $0 + 16 GB GPU | ~1 MB | semi | high |
| Human commission | chosen by ear up front | cleanest (written grant) | exact brief | ~$15–50 hobby, $100–300 pro, each | ~1 MB | project file if requested | low–medium (brief, review) |
| Library: Pro Sensory CC0 set | untested | CC0, cleanest | jazz-chip, not "cantina" | $0; set of 5 exists | 1.3–4 MB each as-is | blob | low |
| Library: AvapXia CC BY set | untested | CC BY, credit | jazz-chip album | $0; 12 tracks | 2–5 min songs, need cutting | blob | medium (cut loops) |
| Library: Swingshot (+ .ftm) | untested | CC0 | best single fit on paper | $0; only one | ~1 MB | **score included** | low |
| In-repo FamiStudio text | person's own | own work | authentic NES | person's time | small text + WAV | **high** | high |
| In-repo Furnace | person's own | own work | widest chip palette | person's time | binary `.fur` + WAV | medium | high |
| Python → MIDI → FluidSynth | person's own | own work; SoundFont licence open | depends on SF2 | person's time | text + WAV | **high** | high |
| Python stdlib synth | person's own | own work | as good as the synth | person's time | text + WAV | **highest** | highest |

**Which remain live.** Udio is out (no downloads). MusicGen is out on its non-commercial weight
licence. The remaining AI services are *legally usable* on a cheap paid tier (ElevenLabs $6,
Suno ~$8, Soundraw $17), but each needs an "AI Generated" tag on itch.io, gives an uncopyrightable
result, and — for a brief that points at one famous cantina tune — carries a provenance risk nobody
can clear; that sits badly with a portfolio piece. AIVA is the interesting exception because its MIDI
export can feed an in-repo chip renderer. Among non-AI options, three stay live: **the library**
(Pro Sensory's CC0 jazz-chip tracks as a zero-cost set, AvapXia's CC BY album as a set that at least
shares a palette, or Swingshot as a single track with its score), which the person can judge by
listening in an afternoon; **a commission** (~$15–50 per loop at hobby rates, ~$100–300 from a
working indie composer), the only way to get *exactly* this style with a clean written licence, and
worth asking for the tracker file so later variations stay in-house; and **composing in the repo with
FamiStudio text or a Python script**, the most reproducible and the most expensive in the person's
time. A sensible order: listen to the library candidates first; if none passes the ear, commission
one track with the project file, then decide whether tracks 2–8 come from the same composer or from
in-repo variations on that file.

---

### Claims not verified

- Suno's current Pro price: pricing page read $8/mo, several third-party pages say $10/mo.
- Suno stems availability by tier (search snippet of help article only).
- Udio's current terms (page did not render); download status rests on secondary reporting.
- Stable Audio hosted: all prices, tiers, licence names (stableaudio.com rendered no text).
- Stable Audio 3.0 release year (page read as 2025; coverage says 20 May 2026).
- Whether the Stability Community Licence's "Powered by Stability AI" notice applies to a game that ships only outputs.
- Mubert: free-tier scope for games, and whether Creator ($14) is commercial (pricing and licence pages disagree); loop support.
- AIVA free tier for games/portfolio: "non-for-profit" is stated, games not named.
- Soundraw: whether jazz or chiptune presets exist.
- Beatoven pricing (page 404); its "perpetual … during the term" wording.
- Google Flow Music free-vs-paid commercial rules; Lyria API price; MusicFX shutdown date (secondary).
- Fiverr gig prices, deliverables, turnaround and commercial-rights add-ons (Fiverr returned 403).
- No Reddit threads read (r/gamedev, r/chiptunes, r/gameDevClassifieds).
- Newgrounds audio portal and its licence terms (403); ccMixter (no readable results).
- Track lengths for most OpenGameArt entries (not listed); nothing listened to.
- "Aug 1 Chiptunes" posting date (page read as 1 Aug 2026; may be misread).
- FamiStudio command-line invocation path on macOS.
- A licence-clean chiptune SoundFont (Musical Artifacts pages 403); "NES 8-Bit Soundfont" CC-BY 3.0 from a search snippet only.

---

## Findings so far (the person, 2026-09-24)

Listening results and rulings from the exploration session, in order:

- **In-repo composition, first take** (two swing chiptune loops from a
  stdlib Python script, melodies through-composed): "not terrible … sound
  like music, but artificial, like the melodies are arbitrary, following a
  style correctly with no real understanding of it."
- **Second take** (same keys, tempos and chords; one motif per section,
  stated, answered, varied, closed; dynamics and articulation added):
  "better in the sense that it sounds more like a composition, but the
  melodies themselves are uninspired." The lead voice — a naive,
  un-band-limited 25 % pulse — was "harsh and grating."
- **Chiptune is not a requirement.** The placeholder track is not chiptune
  either; it uses realistic synth-based instrumentation, and that is the
  preferred direction. The sound problem is a rendering choice (aliasing,
  narrow duty, no filtering), not a limit of composing in the repo.
- **Melody selection is the person's.** The motif-audition approach was
  accepted: many short candidate motifs are rendered, the person picks the
  ones worth building tunes from, so taste enters where it matters most.
- **Style direction.** A "spacey" vibe overall; not strictly swing —
  electronica, ambient or beat-based, is welcome. Each region should be
  very distinct (for example one swing-cantina, one spacey and
  atmospheric, one electronic and beat-based), sharing some guiding
  principle.

Still open: whether real sampled instruments via FluidSynth are wanted
alongside the in-repo synth (needs a Homebrew install and a soundfont
download, both the person's call); which motifs survive the audition; and
the guiding principle that ties the regional tracks together.
