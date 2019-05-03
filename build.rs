extern crate num_cpus;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str;

#[derive(Debug)]
struct Library {
    name: &'static str,
    is_feature: bool,
}

static LIBRARIES: &[Library] = &[
    Library {name: "avcodec", is_feature: true},
    Library {name: "avdevice", is_feature: true},
    Library {name: "avfilter", is_feature: true},
    Library {name: "avformat", is_feature: true},
    Library {name: "avresample", is_feature: true},
    Library {name: "avutil", is_feature: false},
    Library {name: "postproc", is_feature: true},
    Library {name: "swresample", is_feature: true},
    Library {name: "swscale", is_feature: true},
];

fn version() -> String {
    let major: u8 = env::var("CARGO_PKG_VERSION_MAJOR")
        .unwrap()
        .parse()
        .unwrap();
    let minor: u8 = env::var("CARGO_PKG_VERSION_MINOR")
        .unwrap()
        .parse()
        .unwrap();

    format!("{}.{}", major, minor)
}

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn source() -> PathBuf {
    output().join(format!("ffmpeg-{}", version()))
}

fn search() -> PathBuf {
    let mut absolute = env::current_dir().unwrap();
    absolute.push(&output());
    absolute.push("dist");

    absolute
}

fn fetch() -> io::Result<()> {
    let status = try!(
        Command::new("git")
            .current_dir(&output())
            .arg("clone")
            .arg("-b")
            .arg("ts-offset")
            .arg("--depth=1")
            .arg("https://github.com/ngotchac/FFmpeg.git")
            .arg(format!("ffmpeg-{}", version()))
            .status()
    );

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    }
}

fn switch(configure: &mut Command, feature: &str, name: &str) {
    let arg = if env::var("CARGO_FEATURE_".to_string() + feature).is_ok() {
        "--enable-"
    }
    else {
        "--disable-"
    };
    configure.arg(arg.to_string() + name);
}

fn build() -> io::Result<()> {
    let mut configure = Command::new("./configure");
    configure.current_dir(&source());
    configure.arg(format!("--prefix={}", search().to_string_lossy()));

    if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
        configure.arg(format!("--cross-prefix={}-", env::var("TARGET").unwrap()));
    }

    // control debug build
    if env::var("DEBUG").is_ok() {
        configure.arg("--enable-debug");
        configure.arg("--disable-stripping");
    } else {
        configure.arg("--disable-debug");
        configure.arg("--enable-stripping");
    }

    // make it static
    configure.arg("--enable-static");
    configure.arg("--disable-shared");

    configure.arg("--enable-pic");

    macro_rules! enable {
        ($conf:expr, $feat:expr, $name:expr) => (
            if env::var(concat!("CARGO_FEATURE_", $feat)).is_ok() {
                $conf.arg(concat!("--enable-", $name));
            }
        )
    }

    // macro_rules! disable {
    //     ($conf:expr, $feat:expr, $name:expr) => (
    //         if env::var(concat!("CARGO_FEATURE_", $feat)).is_err() {
    //             $conf.arg(concat!("--disable-", $name));
    //         }
    //     )
    // }

    // the binary using ffmpeg-sys must comply with GPL
    switch(&mut configure, "BUILD_LICENSE_GPL", "gpl");

    // the binary using ffmpeg-sys must comply with (L)GPLv3
    switch(&mut configure, "BUILD_LICENSE_VERSION3", "version3");

    // the binary using ffmpeg-sys cannot be redistributed
    switch(&mut configure, "BUILD_LICENSE_NONFREE", "nonfree");

    // configure building libraries based on features
    for lib in LIBRARIES.iter().filter(|lib| lib.is_feature) {
        switch(&mut configure, &lib.name.to_uppercase(), lib.name);
    }

    // configure external SSL libraries
    enable!(configure, "BUILD_LIB_GNUTLS", "gnutls");
    enable!(configure, "BUILD_LIB_OPENSSL", "openssl");

    // configure external filters
    enable!(configure, "BUILD_LIB_FONTCONFIG", "fontconfig");
    enable!(configure, "BUILD_LIB_FREI0R", "frei0r");
    enable!(configure, "BUILD_LIB_LADSPA", "ladspa");
    enable!(configure, "BUILD_LIB_ASS", "libass");
    enable!(configure, "BUILD_LIB_FREETYPE", "libfreetype");
    enable!(configure, "BUILD_LIB_FRIBIDI", "libfribidi");
    enable!(configure, "BUILD_LIB_OPENCV", "libopencv");

    // configure external encoders/decoders
    enable!(configure, "BUILD_LIB_AACPLUS", "libaacplus");
    enable!(configure, "BUILD_LIB_CELT", "libcelt");
    enable!(configure, "BUILD_LIB_DCADEC", "libdcadec");
    enable!(configure, "BUILD_LIB_FAAC", "libfaac");
    enable!(configure, "BUILD_LIB_FDK_AAC", "libfdk-aac");
    enable!(configure, "BUILD_LIB_GSM", "libgsm");
    enable!(configure, "BUILD_LIB_ILBC", "libilbc");
    enable!(configure, "BUILD_LIB_VAZAAR", "libvazaar");
    enable!(configure, "BUILD_LIB_MP3LAME", "libmp3lame");
    enable!(configure, "BUILD_LIB_OPENCORE_AMRNB", "libopencore-amrnb");
    enable!(configure, "BUILD_LIB_OPENCORE_AMRWB", "libopencore-amrwb");
    enable!(configure, "BUILD_LIB_OPENH264", "libopenh264");
    enable!(configure, "BUILD_LIB_OPENH265", "libopenh265");
    enable!(configure, "BUILD_LIB_OPENJPEG", "libopenjpeg");
    enable!(configure, "BUILD_LIB_OPUS", "libopus");
    enable!(configure, "BUILD_LIB_SCHROEDINGER", "libschroedinger");
    enable!(configure, "BUILD_LIB_SHINE", "libshine");
    enable!(configure, "BUILD_LIB_SNAPPY", "libsnappy");
    enable!(configure, "BUILD_LIB_SPEEX", "libspeex");
    enable!(
        configure,
        "BUILD_LIB_STAGEFRIGHT_H264",
        "libstagefright-h264"
    );
    enable!(configure, "BUILD_LIB_THEORA", "libtheora");
    enable!(configure, "BUILD_LIB_TWOLAME", "libtwolame");
    enable!(configure, "BUILD_LIB_UTVIDEO", "libutvideo");
    enable!(configure, "BUILD_LIB_VO_AACENC", "libvo-aacenc");
    enable!(configure, "BUILD_LIB_VO_AMRWBENC", "libvo-amrwbenc");
    enable!(configure, "BUILD_LIB_VORBIS", "libvorbis");
    enable!(configure, "BUILD_LIB_VPX", "libvpx");
    enable!(configure, "BUILD_LIB_WAVPACK", "libwavpack");
    enable!(configure, "BUILD_LIB_WEBP", "libwebp");
    enable!(configure, "BUILD_LIB_X264", "libx264");
    enable!(configure, "BUILD_LIB_X265", "libx265");
    enable!(configure, "BUILD_LIB_AVS", "libavs");
    enable!(configure, "BUILD_LIB_XVID", "libxvid");

    // other external libraries
    enable!(configure, "BUILD_NVENC", "nvenc");

    // configure external protocols
    enable!(configure, "BUILD_LIB_SMBCLIENT", "libsmbclient");
    enable!(configure, "BUILD_LIB_SSH", "libssh");

    // configure misc build options
    enable!(configure, "BUILD_PIC", "pic");

    // run ./configure
    {
        let output = configure
            .output()
            .expect(&format!("{:?} failed", configure));
        if !output.status.success() {
            println!("configure: {}", String::from_utf8_lossy(&output.stdout));

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "configure failed {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }
    }

    // run make
    if !try!(
        Command::new("make")
            .arg("-j")
            .arg(num_cpus::get().to_string())
            .current_dir(&source())
            .status()
    ).success()
    {
        return Err(io::Error::new(io::ErrorKind::Other, "make failed"));
    }

    // run make install
    if !try!(
        Command::new("make")
            .current_dir(&source())
            .arg("install")
            .status()
    ).success()
    {
        return Err(io::Error::new(io::ErrorKind::Other, "make install failed"));
    }

    fs::rename(source().join("ffmpeg"), output().join("ffmpeg"))?;
    fs::rename(source().join("ffprobe"), output().join("ffprobe"))?;

    Ok(())
}

fn main() {
    if fs::metadata(&output().join("ffmpeg")).is_err() {
        fs::create_dir_all(&output())
            .ok()
            .expect("failed to create build directory");
        fetch().unwrap();
        build().unwrap();
    }
    eprintln!("\n\nFinished building FFMpeg at: {:?}\n\n", source());
}
