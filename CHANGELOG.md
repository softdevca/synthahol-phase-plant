# 0.3.1

* Update dependencies and switch to the 2024 edition of Rust.

# 0.3.0

* Partial support the Nonlinear Filter Generator added in Phase Plant 2.1.1.
* Added support for Phase Plant 2.1.3.
* Detect Phase Plant 2.2 presets with their new format.

# 0.2.2 (2023-10-17)

* Effects now remember their effect version so the same version can be written that was read.
* Read more Multipass.
* Decode more modulations.

# 0.2.1 (2023-06-07)

* Successfully read more factory presets.
* Support triggering for the envelope modulator.
* Convert any legacy Aftertouch modulators to Pressure modulators.
* Add mod wheel and modulator output modulation sources.

# 0.2.0 (2023-06-05)

* Use units of measure in many more places for improved type safety.

# 0.1.1 (2023-06-02)

* Recognize more forms of modulation.
* Decode more sections of the file format.
* Modulation "modules" are now called "groups" to avoid confusion with which 
  specific module is being referenced within the group.

# 0.1.0 (2023-05-04)

* Initial release.
