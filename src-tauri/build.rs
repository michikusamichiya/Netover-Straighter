fn main() {
    // // nvEncodeAPI.h からRustバインディングを生成する
    // // パスは C:\nvenc\ に配置したヘッダファイルを参照する
    // let bindings = bindgen::Builder::default()
    //     .header("C:/Users/PC_User/Downloads/Video_Codec_Interface_13.0.37/Video_Codec_Interface_13.0.37/Interface/nvEncodeAPI.h")
    //     // NVENCに必要な型・関数のみ生成する
    //     .allowlist_type("NV_ENC_.*")
    //     .allowlist_function("NvEncodeAPICreateInstance")
    //     .allowlist_var("NVENCAPI_.*")
    //     // Windows特有の型を認識させる
    //     .clang_arg("-x")
    //     .clang_arg("c")
    //     .clang_arg("-D_WIN32")
    //     .clang_arg("-DNOMINMAX")
    //     .generate()
    //     .expect("Failed to generate NVENC bindings");
 
    // // OUT_DIR にバインディングファイルを出力する
    // // OUT_DIR はcargoが自動的に設定する一時ディレクトリ
    // let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    // bindings
    //     .write_to_file(out_path.join("nvenc_bindings.rs"))
    //     .expect("Failed to write NVENC bindings");
 
    // // nvencodeapi.lib のリンク設定
    // // NVIDIAドライバに含まれるDLLへの遅延ロード用スタブライブラリ
    // // ドライバがインストールされていればシステムに存在する
    // println!("cargo:rustc-link-lib=nvencodeapi");

  tauri_build::build()
}
