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
        .expect("`CARGO_PKG_VERSION_MAJOR` is always set in build; qed")
        .parse()
        .expect("`CARGO_PKG_VERSION_MAJOR` is always a number; qed");
    let minor: u8 = env::var("CARGO_PKG_VERSION_MINOR")
        .expect("`CARGO_PKG_VERSION_MINOR` is always set in build; qed")
        .parse()
        .expect("`CARGO_PKG_VERSION_MINOR` is always a numer; qed");

    format!("{}.{}", major, minor)
}

fn output() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` is always set in build; qed"))
}

fn source() -> PathBuf {
    output().join(format!("ffmpeg-{}", version()))
}

fn search() -> PathBuf {
    let mut absolute = env::current_dir().expect("`env::current_dir` always exists in build; qed");
    absolute.push(&output());
    absolute.push("dist");

    absolute
}

fn fetch() -> io::Result<()> {
    if let Ok(meta) = fs::metadata(&source()) {
        if meta.is_dir() {
            return Ok(());
        }
    }
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

// fn switch(configure: &mut Command, feature: &str, name: &str) {
//     let arg = if env::var("CARGO_FEATURE_".to_string() + feature).is_ok() {
//         "--enable-"
//     }
//     else {
//         "--disable-"
//     };
//     configure.arg(arg.to_string() + name);
// }

fn switch(configure: &mut Command, feature: &str, name: &str) {
    if env::var("CARGO_FEATURE_".to_string() + feature).is_ok() {
        configure.arg(format!("--enable-{}", name));
    }
    
}

fn build_libx264() -> io::Result<()> {
    let fetch_status = Command::new("git")
        .current_dir(&output())
        .arg("clone")
        .arg("--depth=1")
        .arg("git://git.videolan.org/x264.git")
        .status();
    let fetch_status = match fetch_status {
        Ok(status) => status,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "x264 fetch failed - make sure git is installed")),
    };
    if !fetch_status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "x264 fetch failed"))
    }

    let configure_status = Command::new("./configure")
        .current_dir(&output().join("x264"))
        .arg("--prefix").arg(search())
        .arg("--bindir").arg(output().join("bin"))
        .arg("--enable-static")
        .arg("--disable-asm")
        .status();
    let configure_status = match configure_status {
        Ok(status) => status,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "x264 configure failed")),
    };
    if !configure_status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "x264 configure failed"))
    }

    let make_status = Command::new("make")
        .current_dir(&output().join("x264"))
        .status();
    let make_status = match make_status {
        Ok(status) => status,
        Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "x264 make failed")),
    };
    if !make_status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "x264 make failed"))
    }

    Ok(())
}

fn build() -> io::Result<()> {
    let mut configure = Command::new("./configure");
    configure.current_dir(&source());
    configure.arg(format!("--prefix={}", search().to_string_lossy()));
    configure.arg(format!("--extra-cflags=\"-I{}\"", search().join("include").to_string_lossy()));
    configure.arg(format!("--extra-ldflags=\"-L{}\"", search().join("lib").to_string_lossy()));

    if env::var("TARGET").expect("`TARGET` is always set in build; qed") != 
        env::var("HOST").expect("`HOST` is always set in build; qed") {
        configure.arg(format!("--cross-prefix={}-", env::var("TARGET").expect("`TARGET` is always set in build; qed")));
    }

    configure.arg("--disable-doc");
    configure.arg("--disable-ffplay");

    configure.arg("--disable-debug");
    configure.arg("--enable-stripping");

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
        eprintln!("Running: {:?}", configure);

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
    // if !try!(
    //     Command::new("make")
    //         .current_dir(&source())
    //         .arg("install")
    //         .status()
    // ).success()
    // {
    //     return Err(io::Error::new(io::ErrorKind::Other, "make install failed"));
    // }

    // Copy over the binaries
    fs::rename(source().join("ffmpeg"), output().join("ffmpeg"))?;
    fs::rename(source().join("ffprobe"), output().join("ffprobe"))?;

    // Remove the source and dist directory
    fs::remove_dir_all(source())?;

    Ok(())
}

fn main() {
    if fs::metadata(&output().join("ffmpeg")).is_err() {
        fs::create_dir_all(&output())
            .ok()
            .expect("failed to create build directory");
        build_libx264().expect("build x264 failed");
        fetch().expect("fetch failed");
        build().expect("build failed");
    }
    eprintln!("\n\nFinished building FFMpeg at: {:?}\n\n", source());
}
