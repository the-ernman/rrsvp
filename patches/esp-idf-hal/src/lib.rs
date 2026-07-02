// Local patch for esp-idf-hal 0.46.2.
// This stub satisfies the workspace [patch.crates-io] requirement so that
// `cargo check -p rrsvp-core -p rrsvp-desktop` works without the full Xtensa toolchain.
//
// For actual embedded builds (rrsvp-embedded), replace this directory with the
// real esp-idf-hal 0.46.2 source patched to rename:
//   rmt_receive_config_t_extra_rmt_receive_flags -> rmt_receive_config_t_extra_flags
//
// Obtain the real source: git clone https://github.com/esp-rs/esp-idf-hal -b v0.46.2 patches/esp-idf-hal
// Then apply the field rename in the relevant bindings file.
