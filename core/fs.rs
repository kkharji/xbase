use anyhow::bail;

#[derive(Debug)]
pub enum Fs {
    /// new file created or deleted or renamed
    DirectoryChange(String),
    /// a file got update
    FileUpdate(String),
    /// Project.yml got modified
    ProjectUpdate,
}

impl TryFrom<Vec<&str>> for Fs {
    type Error = anyhow::Error;

    fn try_from(mut args: Vec<&str>) -> Result<Self, Self::Error> {
        let fs = args.remove(0);
        let target = args.first().unwrap_or(&"").to_string();
        match fs {
            "project_update" => Ok(Self::ProjectUpdate),
            "directory_change" => Ok(Self::DirectoryChange(target)),
            "file_update" => Ok(Self::FileUpdate(target)),
            _ => bail!("Unknown file system messsage: {fs}"),
        }
    }
}

// assert! {
//     matches! {
//         Message::try_from("fs project_update".split(" ").collect::<Vec<&str>>()).unwrap(),
//         Message::Fs(Fs::ProjectUpdate)
//     }
// };

// assert! {
//     matches! {
//         Message::try_from("fs directory_change".split(" ").collect::<Vec<&str>>()).unwrap(),
//         Message::Fs(Fs::DirectoryChange(_))
//     }
// }
