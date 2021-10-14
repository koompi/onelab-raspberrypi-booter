use run_script::{
    ScriptOptions, 
    run_script,
};

pub fn check_password(password: &str) -> (i32, String, String){

    let options = ScriptOptions::new();
    let command = format!("echo {} | sudo -S true", password);
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)

}

pub fn read_disk(password: &str) -> (i32, String, String){

    let options = ScriptOptions::new();
    let command = format!(r#"fdiskOUTPUT=$(echo {} | sudo -S fdisk -l)
rm -rf /tmp/ReadDisk;
while read -r line;
do

#find out whether the line at character 5th to 10th which is being read start with /dev/
if [[ "${{line:5:10}}" == /dev/* ]] ;
then
    
    #filter the line by looking for commas and take the pre-line number 1 and change all spaces that output into underscore and put it into file at /tmp/ReadDisk
    var=$(echo "$line" | awk -F',' '{{printf $1}}');
    alldisks=$(echo "$alldisks|$var");
fi
done <<< "$fdiskOUTPUT" 
echo "$alldisks""#, password);
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)

}

pub fn cleanup_mount_before_run(password: &str, drive: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let command = format!(
        r#"echo {} | sudo -S umount "{}1"; echo password | sudo -S umount "{}2";"#, password, drive, drive
    );
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)

}

pub fn create_partition(password: &str, drive: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let command = format!(
r#"echo {} | sudo -S parted {} mklabel msdos --script;
echo {} | sudo -S parted {} mkpart primary fat32 0% 106M --script;
echo {} | sudo -S parted {} mkpart primary ext4 106M 80% --script;
echo {} | sudo -S parted {} mkpart primary linux-swap 80% 100% --script;
"#, password, drive, password, drive, password, drive, password, drive);
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)

}

pub fn create_format(password: &str, drive: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let command = format!(r#"	echo {} | sudo -S mkfs.ext4 -F "{}2";
echo {} | sudo -S mkfs.vfat -F32 "{}1";echo {} | sudo -S mkswap "{}3";"#, 
    password, drive, password, drive, password, drive);
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)
}

pub fn get_rootuuid(device: &str) -> String {
    let options = ScriptOptions::new();
    let command = format!(r#"ls -lah /dev/disk/by-uuid | grep $(echo "{}2" | awk -F'/' '{{printf $3}}') | awk -F' ' '{{printf $9}}'"#, device);
    let (_code, output, _error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    output
}

pub fn mount_all(password: &str, rootuuid: &str, device: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let mut command = format!(
r#"	echo password | sudo -S rm -rf /tmp/"rootuuid"
echo password | sudo -S mkdir -p /tmp/"rootuuid"
echo password | sudo -S mount "sdcard2" /tmp/"rootuuid"
echo password | sudo -S mkdir /tmp/"rootuuid"/boot
echo password | sudo -S mount "sdcard1" /tmp/"rootuuid"/boot"#
    );
    command = command.replacen("password", password, 5);
    command = command.replacen("sdcard", device, 2);
    command = command.replacen("rootuuid", rootuuid, 5);

    // println!("{}", rootuuid);

    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)
}

pub fn transfer_files(password: &str, rootuuid: &str, filepath: &str, compression_type: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let command: String = match compression_type {
        "lz4" => format!(r#"echo {} | sudo -S tar -C /tmp/"{}" -I lz4 -xvpf {} &>/tmp/{}-log.txt"#, password, rootuuid, filepath, rootuuid),
        "lzo" => format!(r#"echo {} | sudo -S tar -C /tmp/"{}" --lzop -xvpf {} &>/tmp/{}-log.txt"#, password, rootuuid, filepath, rootuuid),
        "gz" => format!(r#"echo {} | sudo -S tar -C /tmp/"{}" -xzvpf {} &>/tmp/{}-log.txt"#,password,rootuuid, filepath, rootuuid),
        _ => format!(r#"echo {} | sudo -S tar -C /tmp/"{}" -xavpf {} &>/tmp/{}-log.txt"#, password, rootuuid, filepath, rootuuid),
    };
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();
    (code, output, error) 
}

pub fn cleanup_mount_after_run(password: &str, rootuuid: &str) -> (i32, String, String) {
    let options = ScriptOptions::new();
    let command = format!(
r#"echo {} | sudo -S umount "/tmp/{}/boot";
echo {} | sudo -S umount "/tmp/{}";
echo {} | sudo -S rm -rf /tmp/"{}";"#, 
    password, rootuuid, password, rootuuid, password, rootuuid);
    let (code, output, error) = run_script!(
        &command,
        &vec![],
        &options
    ). unwrap();

    (code, output, error)
}

