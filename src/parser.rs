use ahqstore_types::{AHQStoreApplication, AndroidAbi, AppRepo, DownloadUrl, InstallerFormat, InstallerOptions, InstallerOptionsAndroid};
use std::{
  collections::HashMap, fs::{self, File}, io::Write
};

use crate::structs::{Metadata, Package};

struct Map {
  entries: usize,
  files: usize,
  c_file: File,
  search: File,
}

impl Map {
  fn new() -> Self {
    let _ = fs::create_dir_all("./db/map");
    let _ = fs::create_dir_all("./db/search");
    let _ = fs::create_dir_all("./db/apps");
    let _ = fs::create_dir_all("./db/dev");
    let _ = fs::create_dir_all("./db/res");

    let mut file = File::create("./db/map/1.json").unwrap();
    let _ = file.write(b"{");

    let mut search = File::create("./db/search/1.json").unwrap();
    let _ = search.write(b"[");

    Self {
      entries: 0,
      files: 1,
      c_file: file,
      search,
    }
  }

  fn close_file(&mut self) {
    let _ = self.search.write_all(b"]");
    let _ = self.search.flush();
    let _ = self.c_file.write_all(b"}");
    let _ = self.c_file.flush();
  }

  fn new_file(&mut self) {
    self.files += 1;
    self.entries = 0;
    self.close_file();

    let mut map = File::create("./db/map/1.json").unwrap();
    let _ = map.write(b"{");

    let mut search = File::create("./db/map/1.json").unwrap();
    let _ = search.write(b"[");

    self.c_file = map;
    self.search = search;
  }

  fn add_author(&mut self, author: &str, app_id: &str) {
    let file = format!("./db/dev/{}", author);
    let mut val = fs::read_to_string(&file).unwrap_or("".to_string());
    val.push_str(&format!("f:{}\n", &app_id));

    let _ = fs::write(&file, val);
  }

  fn add(&mut self, mut app: AHQStoreApplication) {
    if self.entries >= 100_000 {
      self.new_file();
    }
    println!("{}", self.entries);
    if self.entries > 0 {
      let _ = self.c_file.write(b",");
      let _ = self.search.write(b",");
    }

    self.add_author(&app.authorId, &app.appId);
    self.entries += 1;

    let _ = self
      .c_file
      .write(format!("\"{}\":\"f:{}\"", app.appDisplayName, app.appId).as_bytes());
    let _ = self.search.write(
      format!(
        "{{\"name\": {:?}, \"title\": {:?}, \"id\": {:?}}}",
        app.appDisplayName, app.appShortcutName, format!("f:{}", app.appId)
      )
      .as_bytes(),
    );

    let (app_str, res) = app.export();

    let path = format!("./db/apps/{}.json", &app.appId);

    let _ = fs::create_dir_all(format!("./db/res/{}", &app.appId));

    for (id, bytes) in res {
      let _ = fs::write(format!("./db/res/{}/{}", &app.appId, id), bytes);
    }

    app.appId = format!("f:{}", app.appId);
    app.authorId = format!("f:{}", app.authorId);

    println!("✅ Adding {}", &app.appId);

    let _ = fs::write(path, app_str);
  }

  fn finish(mut self) {
    self.close_file();

    let _ = fs::write("./db/total", self.files.to_string());
  }
}

pub fn parser(meta: Metadata) {
  println!("⏲️ Please wait...");
  let _ = fs::remove_dir_all("./db");
  let _ = fs::create_dir_all("./db");

  let mut map = Map::new();

  for (id, meta) in meta.packages {
    if let Some(mut data) = get_imp_data(meta) {
      map.add(AHQStoreApplication {
        appId: id,
        appDisplayName: data.name.clone(),
        appShortcutName: data.name,
        authorId: format!("fdroid"),
        description: format!("{}\n{}", data.summary.unwrap_or("".into()), data.desc),
        displayImages: vec![],
        downloadUrls: {
          let mut m = HashMap::new();

          m.insert(1, DownloadUrl {
            asset: "".into(),
            installerType: InstallerFormat::AndroidApkZip,
            url: data.download
          });

          m
        },
        install: InstallerOptions {
          linux: None,
          linuxArm64: None,
          linuxArm7: None,
          win32: None,
          winarm: None,
          android: Some(InstallerOptionsAndroid {
            assetId: 1,
            abi: data.abi,
            min_sdk: data.min
          })
        },
        license_or_tos: data.license,
        releaseTagName: "".into(),
        repo: AppRepo { author: data.author, repo: data.repo.clone().unwrap_or("".into()) },
        resources: {
          let mut res = HashMap::new();

          res.insert(0, unsafe { data.icon.as_mut_vec() }.clone());

          Some(res)
        },
        site: None,
        source: data.repo,
        verified: false,
        version: data.version,
      });
    } else {
      println!("⚠️ Could not read metadata of {id}");
    }
  }

  map.finish();
  println!("✅ Done!");
}

pub struct ImpData {
  pub author: String,
  pub desc: String,
  pub icon: String,
  pub name: String,
  pub repo: Option<String>,
  pub summary: Option<String>,
  pub version: String,
  pub download: String,
  pub license: Option<String>,
  pub min: u32,
  pub abi: Vec<AndroidAbi>
}

fn get_imp_data(mut meta: Package) -> Option<ImpData> {
  let author = meta.metadata.author;
  let desc = meta.metadata.description?.remove("en-US")?;
  
  let icon = (|| {
    Some(format!("https://f-droid.org/repo{}", meta.metadata.icon?.remove("en-US")?.name?))
  })().unwrap_or("https://f-droid.org/assets/ic_repo_app_default_KNN008Z2K7VNPZOFLMTry3JkfFYPxVGDopS1iwWe5wo=.png".into());

  let name = meta.metadata.name.remove("en-US")?;
  let repo = meta.metadata.repo;
  let summary = meta.metadata.summary.and_then(|mut s| s.remove("en-US"));
  let license = meta.metadata.license;
  
  let mut vint = 0;
  let mut ver = String::from("");
  let mut download = String::from("");
  let mut abi = vec![];

  let mut min = 0;

  for (v, data) in meta.versions {
    if let Some(x) = data.manifest.info {
      let abis = data.manifest.abi.unwrap_or_else(|| {
        vec![
          "arm64-v8a".into(),
          "armeabi-v7a".into(),
          "x86".into(),
          "x86_64".into()
        ]
      });

      let ve = data.manifest.version;
      if ve >= vint {
        vint = ve;
        ver = v;
        download = format!("https://f-droid.org/repo{}", data.file.name);
        min = x.min.unwrap_or(30);

        abi = abis.into_iter()
          .map(|x| match x.as_str() {
            "arm64-v8a" => AndroidAbi::Aarch64,
            "armeabi-v7a" => AndroidAbi::Armv7,
            "x86" => AndroidAbi::X86,
            "x86_64" => AndroidAbi::X64,
            _ => unreachable!()
          })
          .collect();
      }
    }
  }

  if download.is_empty() {
    return None;
  }

  Some(
    ImpData { author, desc, license, icon, name, repo, summary, version: ver, download, min, abi }
  )
}