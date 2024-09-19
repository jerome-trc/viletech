#ifndef ZMSX_H
#define ZMSX_H

#include <stdbool.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdarg.h>

#if __STDC_VERSION__ >= 202000L || (defined(__cplusplus) && __cplusplus >= 201703L)
#define ZMSX_NODISCARD [[nodiscard]]
#else
#define ZMSX_NODISCARD
#endif

struct SoundDecoder;	// Anonymous to the client.

// These constants must match the corresponding values of the Windows headers
// to avoid readjustment in the native Windows device's playback functions
// and should not be changed.
typedef enum zmsx_MidiDeviceClass_ {
	zmsx_devcls_midiport = 1,
	zmsx_devcls_synth,
	zmsx_devcls_sqsynth,
	zmsx_devcls_fmsynth,
	zmsx_devcls_mapper,
	zmsx_devcls_wavetable,
	zmsx_devcls_swsynth
} zmsx_MidiDeviceClass;

typedef enum zmsx_MidiType_ {
	MIDI_NOTMIDI,
	MIDI_MIDI,
	MIDI_HMI,
	MIDI_XMI,
	MIDI_MUS,
	MIDI_MIDS
} zmsx_MidiType;

typedef enum zmsx_MidiDevice_ {
	MDEV_DEFAULT = -1,
	MDEV_STANDARD = 0,
	MDEV_OPL = 1,
	MDEV_SNDSYS = 2,
	MDEV_TIMIDITY = 3,
	MDEV_FLUIDSYNTH = 4,
	MDEV_GUS = 5,
	MDEV_WILDMIDI = 6,
	MDEV_ADL = 7,
	MDEV_OPN = 8,

	MDEV_COUNT
} zmsx_MidiDevice;

typedef enum zmsx_SoundFontTypes_ {
	SF_SF2 = 1,
	SF_GUS = 2,
	SF_WOPL = 4,
	SF_WOPN = 8
} zmsx_SoundFontTypes;

typedef struct zmsx_SoundStreamInfo_ {
	/// If 0, the song doesn't use streaming
	/// but plays through a different interface.
	int mBufferSize;
	int mSampleRate;
	/// If negative, 16 bit integer format is used instead of floating point.
	int mNumChannels;
} zmsx_SoundStreamInfo;

typedef enum zmsx_SampleType_ {
	SampleType_UInt8,
	SampleType_Int16,
	SampleType_Float32
} zmsx_SampleType;

typedef enum zmsx_ChannelConfig_ {
	ChannelConfig_Mono,
	ChannelConfig_Stereo
} zmsx_ChannelConfig;

typedef struct zmsx_SoundStreamInfoEx_ {
	/// If 0, the song doesn't use streaming but plays through a different interface.
	int mBufferSize;
	int mSampleRate;
	zmsx_SampleType mSampleType;
	zmsx_ChannelConfig mChannelConfig;
} zmsx_SoundStreamInfoEx;

typedef enum zmsx_IntConfigKey_ {
	zmusic_adl_chips_count,
	zmusic_adl_emulator_id,
	zmusic_adl_run_at_pcm_rate,
	zmusic_adl_fullpan,
	zmusic_adl_bank,
	zmusic_adl_use_custom_bank,
	zmusic_adl_volume_model,

	zmusic_fluid_reverb,
	zmusic_fluid_chorus,
	zmusic_fluid_voices,
	zmusic_fluid_interp,
	zmusic_fluid_samplerate,
	zmusic_fluid_threads,
	zmusic_fluid_chorus_voices,
	zmusic_fluid_chorus_type,

	zmusic_opl_numchips,
	zmusic_opl_core,
	zmusic_opl_fullpan,

	zmusic_opn_chips_count,
	zmusic_opn_emulator_id,
	zmusic_opn_run_at_pcm_rate,
	zmusic_opn_fullpan,
	zmusic_opn_use_custom_bank,

	zmusic_gus_dmxgus,
	zmusic_gus_midi_voices,
	zmusic_gus_memsize,

	zmusic_timidity_modulation_wheel,
	zmusic_timidity_portamento,
	zmusic_timidity_reverb,
	zmusic_timidity_reverb_level,
	zmusic_timidity_chorus,
	zmusic_timidity_surround_chorus,
	zmusic_timidity_channel_pressure,
	zmusic_timidity_lpf_def,
	zmusic_timidity_temper_control,
	zmusic_timidity_modulation_envelope,
	zmusic_timidity_overlap_voice_allow,
	zmusic_timidity_drum_effect,
	zmusic_timidity_pan_delay,
	zmusic_timidity_key_adjust,

	zmusic_wildmidi_reverb,
	zmusic_wildmidi_enhanced_resampling,

	zmusic_snd_midiprecache,

	zmusic_mod_samplerate,
	zmusic_mod_volramp,
	zmusic_mod_interp,
	zmusic_mod_autochip,
	zmusic_mod_autochip_size_force,
	zmusic_mod_autochip_size_scan,
	zmusic_mod_autochip_scan_threshold,

	zmusic_snd_streambuffersize,

	zmusic_snd_mididevice,
	zmusic_snd_outputrate,

	NUM_ZMUSIC_INT_CONFIGS
} zmsx_IntConfigKey;

typedef enum zmsx_FloatConfigKey_ {
	zmusic_fluid_gain = 1000,
	zmusic_fluid_reverb_roomsize,
	zmusic_fluid_reverb_damping,
	zmusic_fluid_reverb_width,
	zmusic_fluid_reverb_level,
	zmusic_fluid_chorus_level,
	zmusic_fluid_chorus_speed,
	zmusic_fluid_chorus_depth,

	zmusic_timidity_drum_power,
	zmusic_timidity_tempo_adjust,
	zmusic_timidity_min_sustain_time,

	zmusic_gme_stereodepth,
	zmusic_mod_dumb_mastervolume,

	zmusic_snd_musicvolume,
	zmusic_relative_volume,
	zmusic_snd_mastervolume,

	NUM_FLOAT_CONFIGS
} zmsx_FloatConfigKey;

typedef enum zmsx_StringConfigKey_ {
	zmusic_adl_custom_bank = 2000,
	zmusic_fluid_lib,
	zmusic_fluid_patchset,
	zmusic_opn_custom_bank,
	zmusic_gus_config,
	zmusic_gus_patchdir,
	zmusic_timidity_config,
	zmusic_wildmidi_config,

	NUM_STRING_CONFIGS
} zmsx_StringConfigKey;

typedef struct zmsx_CustomReader_ {
	void* handle;
	char* (*gets)(struct zmsx_CustomReader_* handle, char* buff, int n);
	long (*read)(struct zmsx_CustomReader_* handle, void* buff, int32_t size);
	long (*seek)(struct zmsx_CustomReader_* handle, long offset, int whence);
	long (*tell)(struct zmsx_CustomReader_* handle);
	void (*close)(struct zmsx_CustomReader_* handle);
} zmsx_CustomReader;

typedef struct zmsx_MidiOutDevice_ {
	char *Name;
	int ID;
	int Technology;
} zmsx_MidiOutDevice;

typedef enum zmsx_MessageSeverity_ {
	ZMUSIC_MSG_VERBOSE = 1,
	ZMUSIC_MSG_DEBUG = 5,
	ZMUSIC_MSG_NOTIFY = 10,
	ZMUSIC_MSG_WARNING = 50,
	ZMUSIC_MSG_ERROR = 100,
	ZMUSIC_MSG_FATAL = 666,
} zmsx_MessageSeverity;

typedef struct zmsx_Callbacks_ {
	/// Callbacks the client can install to capture messages from the backends
	/// or to provide sound font data.

	void (*MessageFunc)(zmsx_MessageSeverity severity, const char* msg);
	// The message callbacks are optional, without them the output goes to stdout.

	/// Retrieves the path to a soundfont identified by an identifier. Only needed if the client virtualizes the sound font names
	const char *(*PathForSoundfont)(const char *name, int type);

	// The sound font callbacks are for allowing the client to customize sound font management and they are optional.
	// They only need to be defined if the client virtualizes the sound font management and doesn't pass real paths to the music code.
	// Without them only paths to real files can be used. If one of these gets set, all must be set.

	/// This opens a sound font. Must return a handle with which the sound font's content can be read.
	void *(*OpenSoundFont)(const char* name, int type);

	/// Opens a file in the sound font. For GUS patch sets this will try to open each patch with this function.
	/// For other formats only the sound font's actual name can be requested.
	/// When passed NULL this must open the Timidity config file, if this is requested for an SF2 sound font it should be synthesized.
	zmsx_CustomReader* (*SF_OpenFile)(void* handle, const char* fn);

	/// Adds a path to the list of directories in which files must be looked for.
	void (*SF_AddToSearchPath)(void* handle, const char* path);

	// Closes the sound font reader.
	void (*SF_Close)(void* handle);

	/// Used to handle client-specific path macros.
	/// If not set, the path may not contain any special tokens that may need expansion.
	const char *(*NicePath)(const char* path);
} zmsx_Callbacks;

typedef enum zmsx_VarType_ {
	ZMUSIC_VAR_INT,
	ZMUSIC_VAR_BOOL,
	ZMUSIC_VAR_FLOAT,
	ZMUSIC_VAR_STRING,
} zmsx_VarType;

typedef struct zmsx_Setting_ {
	const char* name;
	int identifier;
	zmsx_VarType type;
	float defaultVal;
	const char* defaultString;
} zmsx_Setting;

#ifndef ZMSX_HPP
#if defined(_MSC_VER) && !defined(ZMSX_STATIC)
#define DLL_IMPORT _declspec(dllimport)
#else // if defined(_MSC_VER) && !defined(ZMSX_STATIC)
#define DLL_IMPORT
#endif // if defined(_MSC_VER) && !defined(ZMSX_STATIC)

// Note that the internal 'class' definitions are not C compatible!
typedef struct _ZMusic_MidiSource_Struct { int zm1; } *zmsx_MidiSource;
typedef struct _ZMusic_MusicStream_Struct { int zm2; } *zmsx_MusicStream;
struct SoundDecoder;

#endif // ifndef ZMSX_HPP

#ifndef ZMUSIC_NO_PROTOTYPES

#ifdef __cplusplus
extern "C" {
#endif

	DLL_IMPORT const char* zmsx_get_last_error(void);

	/// Sets callbacks for functionality that the client needs to provide.
	DLL_IMPORT void zmsx_set_callbacks(const zmsx_Callbacks* callbacks);

	/// Sets GenMidi data for OPL playback.
	/// If this isn't provided the OPL synth will not work.
	DLL_IMPORT void zmsx_set_genmidi(const uint8_t* data);

	/// Set default bank for OPN. Without this OPN only works with custom banks.
	DLL_IMPORT void zmsx_set_wgopn(const void* data, unsigned len);

	/// Set DMXGUS data for running the GUS synth in actual GUS mode.
	DLL_IMPORT void zmsx_set_dmxgus(const void* data, unsigned len);

	/// Returns an array with all available configuration options,
	/// terminated with an empty entry where all elements are 0.
	DLL_IMPORT const zmsx_Setting* zmsx_get_config(void);

	/// These exports are needed by the MIDI dumpers which need to remain on the client side
	/// because they need access to the client's file system.
	DLL_IMPORT zmsx_MidiType zmsx_identify_midi_type(uint32_t* id, int size);

	DLL_IMPORT zmsx_MidiSource
		zmsx_create_midi_source(const uint8_t* data, size_t length, zmsx_MidiType miditype);

	DLL_IMPORT bool zmsx_midi_dump_wave(
		zmsx_MidiSource source,
		zmsx_MidiDevice devtype,
		const char* devarg,
		const char* outname,
		int subsong,
		int samplerate
	);

	DLL_IMPORT zmsx_MusicStream
		zmsx_open_song(zmsx_CustomReader* reader, zmsx_MidiDevice device, const char* args);

	DLL_IMPORT zmsx_MusicStream
		zmsx_open_song_file(const char* filename, zmsx_MidiDevice device, const char* args);

	DLL_IMPORT zmsx_MusicStream zmsx_open_song_mem(
		const void* mem,
		size_t size,
		zmsx_MidiDevice device,
		const char* Args
	);

	DLL_IMPORT zmsx_MusicStream zmsx_open_song_cd(int track, int cdid);

	DLL_IMPORT bool zmsx_fill_stream(zmsx_MusicStream stream, void* buff, int len);

	DLL_IMPORT bool zmsx_start(zmsx_MusicStream song, int subsong, bool loop);

	DLL_IMPORT void zmsx_pause(zmsx_MusicStream song);

	DLL_IMPORT void zmsx_resume(zmsx_MusicStream song);

	DLL_IMPORT void zmsx_update(zmsx_MusicStream song);

	DLL_IMPORT bool zmsx_is_playing(zmsx_MusicStream song);

	DLL_IMPORT void zmsx_stop(zmsx_MusicStream song);

	DLL_IMPORT void zmsx_close(zmsx_MusicStream song);

	DLL_IMPORT bool zmsx_set_subsong(zmsx_MusicStream song, int subsong);

	DLL_IMPORT bool zmsx_is_looping(zmsx_MusicStream song);

	DLL_IMPORT int zmsx_get_device_type(zmsx_MusicStream song);

	DLL_IMPORT bool zmsx_is_midi(zmsx_MusicStream song);

	DLL_IMPORT void zmsx_volume_changed(zmsx_MusicStream song);

	DLL_IMPORT bool
		zmsx_write_smf(zmsx_MidiSource source, const char* fn, int looplimit);

	DLL_IMPORT void zmsx_get_stream_info(zmsx_MusicStream song, zmsx_SoundStreamInfo* info);

	DLL_IMPORT void zmsx_get_stream_info_ex(
		zmsx_MusicStream song,
		zmsx_SoundStreamInfoEx* info
	);

	// Configuration interface. The return value specifies if a music restart is needed.
	// RealValue should be written back to the CVAR or whatever other method the client uses to store configuration state.

	DLL_IMPORT bool zmsx_config_set_int(
		zmsx_IntConfigKey key,
		zmsx_MusicStream song,
		int value,
		int* pRealValue
	);

	DLL_IMPORT bool zmsx_config_set_float(
		zmsx_FloatConfigKey key,
		zmsx_MusicStream song,
		float value,
		float* pRealValue
	);

	DLL_IMPORT bool zmsx_config_set_string(
		zmsx_StringConfigKey key,
		zmsx_MusicStream song,
		const char* value
	);

	DLL_IMPORT const char* zmsx_get_stats(zmsx_MusicStream song);

	DLL_IMPORT struct SoundDecoder* zmsx_create_decoder(
		const uint8_t* data,
		size_t size,
		bool isstatic
	);

	DLL_IMPORT void zmsx_sounddecoder_get_info(
		struct SoundDecoder* decoder,
		int* samplerate,
		zmsx_ChannelConfig* chans,
		zmsx_SampleType* type
	);

	DLL_IMPORT size_t
		zmsx_sounddecoder_read(struct SoundDecoder* decoder, void* buffer, size_t length);

	DLL_IMPORT void zmsx_sounddecoder_close(struct SoundDecoder* decoder);

	DLL_IMPORT void zmsx_find_loop_tags(
		const uint8_t* data,
		size_t size,
		uint32_t* start,
		bool* startass,
		uint32_t* end,
		bool* endass
	);

	// The rest of the decoder interface is only useful for streaming music.

	DLL_IMPORT const zmsx_MidiOutDevice *zmsx_get_midi_devices(int *pAmount);

	DLL_IMPORT int zmsx_get_adl_banks(const char* const** pNames);

	// Direct access to the CD drive.
	// Stops playing the CD
	DLL_IMPORT void zmsx_cd_stop(void);

	// Pauses CD playing
	DLL_IMPORT void zmsx_cd_pause(void);

	// Resumes CD playback after pausing
	DLL_IMPORT bool zmsx_cd_resume(void);

	// Eject the CD tray
	DLL_IMPORT void zmsx_cd_eject(void);

	// Close the CD tray
	DLL_IMPORT bool zmsx_cd_uneject(void);

	// Closes a CD device previously opened with CD_Init
	DLL_IMPORT void zmsx_cd_close(void);

	DLL_IMPORT bool zmsx_cd_enable(const char* drive);

#ifdef __cplusplus
}

inline bool ChangeMusicSetting(
	zmsx_IntConfigKey key,
	zmsx_MusicStream song,
	int value,
	int* pRealValue = nullptr
) {
	return zmsx_config_set_int(key, song, value, pRealValue);
}

inline bool ChangeMusicSetting(
	zmsx_FloatConfigKey key,
	zmsx_MusicStream song,
	float value,
	float* pRealValue = nullptr
) {
	return zmsx_config_set_float(key, song, value, pRealValue);
}

inline bool ChangeMusicSetting(
	zmsx_StringConfigKey key,
	zmsx_MusicStream song,
	const char* value
) {
	return zmsx_config_set_string(key, song, value);
}

#endif // ifdef __cplusplus
#endif // ifndef ZMUSIC_NO_PROTOTYPES

// Function typedefs for run-time linking.

typedef const char* (*pfn_zmsx_get_last_error)(void);

typedef void (*pfn_zmsx_set_callbacks)(const zmsx_Callbacks* callbacks);

typedef void (*pfn_zmsx_set_genmidi)(const uint8_t* data);

typedef void (*pfn_zmsx_set_wgopn)(const void* data, unsigned len);

typedef void (*pfn_zmsx_set_dmxgus)(const void* data, unsigned len);

typedef const zmsx_Setting* (*pfn_zmsx_get_config)();

typedef zmsx_MidiType (*pfn_zmsx_identify_midi_type)(uint32_t* id, int size);

typedef zmsx_MidiSource (*pfn_zmsx_create_midi_source)(
	const uint8_t* data,
	size_t length,
	zmsx_MidiType miditype
);

typedef bool (*pfn_zmsx_midi_dump_wave)(
	zmsx_MidiSource source,
	zmsx_MidiDevice devtype,
	const char* devarg,
	const char* outname,
	int subsong,
	int samplerate
);

typedef zmsx_MusicStream (*pfn_zmsx_open_song)(
	zmsx_CustomReader* reader, zmsx_MidiDevice device, const char* Args
);

typedef zmsx_MusicStream (*pfn_zmsx_open_song_file)(
	const char* filename, zmsx_MidiDevice device, const char* Args
);

typedef zmsx_MusicStream (*pfn_zmsx_open_song_mem)(
	const void* mem, size_t size, zmsx_MidiDevice device, const char* Args
);

typedef zmsx_MusicStream (*pfn_zmsx_open_song_cd)(int track, int cdid);

typedef bool (*pfn_zmsx_fill_stream)(
	zmsx_MusicStream stream,
	void* buff,
	int len
);

typedef bool (*pfn_zmsx_start)(
	zmsx_MusicStream song,
	int subsong,
	bool loop
);

typedef void (*pfn_zmsx_pause)(zmsx_MusicStream song);

typedef void (*pfn_zmsx_resume)(zmsx_MusicStream song);

typedef void (*pfn_zmsx_update)(zmsx_MusicStream song);

typedef bool (*pfn_zmsx_is_playing)(zmsx_MusicStream song);

typedef void (*pfn_zmsx_stop)(zmsx_MusicStream song);

typedef void (*pfn_zmsx_close)(zmsx_MusicStream song);

typedef bool (*pfn_zmsx_set_subsong)(zmsx_MusicStream song, int subsong);

typedef bool (*pfn_zmsx_is_looping)(zmsx_MusicStream song);

typedef bool (*pfn_zmsx_is_midi)(zmsx_MusicStream song);

typedef void (*pfn_zmsx_volume_changed)(zmsx_MusicStream song);

typedef bool (*pfn_zmsx_write_smf)(
	zmsx_MidiSource source,
	const char* fn,
	int looplimit
);

typedef void (*pfn_zmsx_get_stream_info)(zmsx_MusicStream song, zmsx_SoundStreamInfo* info);

typedef void (*pfn_zmsx_get_stream_info_ex)(
	zmsx_MusicStream song,
	zmsx_SoundStreamInfoEx* info
);

typedef bool (*pfn_zmsx_config_set_int)(
	zmsx_IntConfigKey key,
	zmsx_MusicStream song,
	int value,
	int* pRealValue
);

typedef bool (*pfn_zmsx_config_set_float)(
	zmsx_FloatConfigKey key,
	zmsx_MusicStream song,
	float value,
	float* pRealValue
);

typedef bool (*pfn_zmsx_config_set_string)(
	zmsx_StringConfigKey key,
	zmsx_MusicStream song,
	const char* value
);

typedef const char* (*pfn_zmsx_get_stats)(zmsx_MusicStream song);

typedef struct SoundDecoder* (*pfn_zmsx_create_decoder)(
	const uint8_t* data,
	size_t size,
	bool isstatic
);

typedef void (*pfn_zmsx_sounddecoder_get_info)(
	struct SoundDecoder* decoder,
	int* samplerate,
	zmsx_ChannelConfig* chans,
	zmsx_SampleType* type
);

typedef size_t (*pfn_zmsx_sounddecoder_read)(
	struct SoundDecoder* decoder,
	void* buffer,
	size_t length
);

typedef void (*pfn_zmsx_sounddecoder_close)(struct SoundDecoder* decoder);

typedef void (*pfn_zmsx_find_loop_tags)(
	const uint8_t* data,
	size_t size,
	uint32_t* start,
	bool* startass,
	uint32_t* end,
	bool* endass
);

typedef const zmsx_MidiOutDevice* (*pfn_zmsx_get_midi_devices)(int* pAmount);

#endif
