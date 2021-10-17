use fltk::{app, button::Button, enums, dialog::{alert, password, FileDialogType, FileDialog}, frame::Frame, menu::Choice, misc::Progress, prelude::*, window::Window};
use std::{sync::mpsc, thread, time::Duration};

mod linux;

fn set_progress_bar(progress_bar: &mut Progress, value: f64, display_text: &str){

    progress_bar.set_value(value);
    progress_bar.set_label(&format!("{}% {}", value, display_text));
    app::check();

}

fn stop(start_button: &mut Button, browse_button: &mut Button, choice_box: &mut Choice){

    start_button.deactivate();
    browse_button.deactivate();
    choice_box.deactivate();

}

fn start(start_button: &mut Button, browse_button: &mut Button, choice_box: &mut Choice){

    start_button.activate();
    browse_button.activate();
    choice_box.activate();

}


fn browse_file(file_frame: &mut Frame) -> FileDialog {
    let mut file_box = FileDialog::new(FileDialogType::BrowseFile);

    file_box.set_directory("/").unwrap();
    file_box.set_title("Select File Boot");
    file_box.set_filter("*.{lzo,lz4,gz}");
    file_box.show();

    file_frame.set_label(format!("Selected File: {}", file_box.filename().as_path().to_str().unwrap()).as_str());
    file_frame.set_label_color(enums::Color::Black);
    
    file_box
}

fn ask_pass() -> String{
    let mut current_password: String;
    loop {
        let password_box = password(650, 420, "Enter Password: ", "");
        current_password = password_box.unwrap();

        let (code, _output, _error) = linux::check_password(&current_password);

        match code {
            0  => break,
            _ => alert(650, 420, "Error: Insufficient Permission OR Incorrect Username Password")
        }
    }
    current_password
}

fn main() {
    // Spawn an App
    let app = app::App::default()
    .with_scheme(app::Scheme::Gtk);

    // Spawn a Channel
    let (fltk_message_sender, fltk_message_reciever) = app::channel::<&str>();


    // Spawn a Window
    let mut wind = Window::default()
        .with_pos(350, 250)
        .with_size(950,500)
        .with_label("Onelab RaspberryPi Booter");
    wind.set_color(enums::Color::White);

    // Spawn a Title Frame
    Frame::default()
        .with_pos(400, 10)
        .with_size(150, 100)
        .with_label("Onelab RaspberryPi Booter")
        .set_label_size(40);

    // Spawn a DropDown Choice Box
    let mut choice_box = Choice::default()
        .with_pos(310, 150)
        .with_size(375, 50)
        .with_label("Select SD Card: ");
    choice_box.set_color(enums::Color::White);

    // Spawn a File Name Frame
    let mut file_frame = Frame::default()
        .with_pos(390, 215)
        .with_size(150, 100)
        .with_label("File Boot: missing");
    file_frame.set_label_size(15);
    file_frame.set_label_color(enums::Color::Red);
 
    // Spawn a Browse Button
    let mut browse_button = Button::default()
        .with_label("Browse")
        .with_pos(340,  310) 
        .with_size(100, 50);
    browse_button.emit(fltk_message_sender, "browse_go");
    browse_button.set_label_color(enums::Color::White);
    browse_button.set_color(enums::Color::from_rgb(148, 0, 211));

    // Spawn a Button Start
    let mut start_button = Button::default()
        .with_label("Start")
        .with_pos(525,  310) 
        .with_size(100, 50);
    start_button.emit(fltk_message_sender, "start_go");
    start_button.set_color(enums::Color::Red);
    start_button.set_label_color(enums::Color::White);

    // Spawn a hidden Progress Bar
    let mut progress_bar = Progress::default()
    .with_pos(70, 410)
    .with_size(800, 25);
    progress_bar.set_maximum(100.0);
    progress_bar.set_minimum(0.0);
    progress_bar.set_color(enums::Color::White);
    progress_bar.set_selection_color(enums::Color::DarkBlue);
    progress_bar.hide();

    // Spawn a Struct for Browse File
    let mut file_box: FileDialog = FileDialog::new(FileDialogType::BrowseFile);

    // Ask for password from User and Verify
    let current_password: String = ask_pass();

    // Collecting all Drives Data 
    let (_code, output, _error) = linux::read_disk(&current_password);
    let splited_output = output.split('|').collect::<Vec<&str>>();

    for i in 0..splited_output.len() {
        let edited_disk = &splited_output[i].replace("Disk ", "");
        choice_box.add_choice(&edited_disk);
    }

    // Show the Window and Refresh the app
    wind.end();
    wind.show();
    app::check();

    while app.wait() {
        if let Some(msg) = fltk_message_reciever.recv() {
            if msg == "start_go"{
                stop(&mut start_button, &mut browse_button, &mut choice_box);
                let choice_output = choice_box.choice().unwrap();
                let choice_splited_vector = choice_output.split(":").collect::<Vec<&str>>();
                let actual_disk = choice_splited_vector[0].to_owned();
                let filepath = file_box.filename().as_path().to_str().unwrap().to_string();

                progress_bar.show();
                app::check();
                
                set_progress_bar(&mut progress_bar, 5.0, "Running Pre-Installation Process");
                let (_code, _output, _error ) = linux::cleanup_mount_before_run(&current_password, &actual_disk);

                set_progress_bar(&mut progress_bar, 10.0, "Creating Partitions for Root, Boot and Swap");
                let (_code, _output, _error) = linux::create_partition(&current_password, &actual_disk);

                set_progress_bar(&mut progress_bar, 15.0, "Setting Format Type for Root, Boot, and Swap");

                let clone_current_password = current_password.clone();
                let clone_actual_disk = actual_disk.clone();

                let (thread_message_sender, thread_message_reciever) = mpsc::channel::<&str>();
                thread::spawn(move || {
                    thread_message_sender.send("creating format starting").unwrap();
                    let (_code, _output, _error) = linux::create_format(&clone_current_password, &clone_actual_disk);
                    thread_message_sender.send("creating format finished").unwrap();
                });

                let mut counter: f64 = 15.0;

                loop {
                    if thread_message_reciever.try_recv().unwrap_or("") == "creating format finished"{
                        break;
                    }
                    else {
                        if counter < 25.0 {
                            counter+=1.0;
                        }
                        else {
                            counter = counter;
                        }

                        set_progress_bar(&mut progress_bar, counter, "Setting Format Type for Root, Boot, and Swap");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Setting Format Type for Root, Boot, and Swap");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Setting Format Type for Root, Boot, and Swap...");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Setting Format Type for Root, Boot, and Swap...");
                        thread::sleep(Duration::from_millis(500));
                    }                   
                }

                let rootuuid: String = linux::get_rootuuid(&actual_disk);  
                
                set_progress_bar(&mut progress_bar, 25.0, "Mounting Newly Created Partitions");
                let (_code, _output, _error) = linux::mount_all(&current_password, &rootuuid, &actual_disk);

                let clone_rootuuid = rootuuid.clone();
                let clone_current_password = current_password.clone();
                let clone_filepath = filepath.clone();

                let (thread_message_sender, thread_message_reciever) = mpsc::channel::<&str>();
                set_progress_bar(&mut progress_bar,40.0, "Installing Systems");
                thread::spawn(move || {
                    thread_message_sender.send("transfering file starting").unwrap();
                    let compression_type_vector = clone_filepath.split(".").collect::<Vec<&str>>();
                    let compression_type = compression_type_vector[compression_type_vector.len()-1];
                    let (_code, _output, _error) = linux::transfer_files(&clone_current_password, &clone_rootuuid, &clone_filepath, &compression_type);
                    thread_message_sender.send("transfering file finished").unwrap();
                });

                counter = 40.0;

                loop {
                    if thread_message_reciever.try_recv().unwrap_or("") == "transfering file finished"{
                        break;
                    }
                    else {
                        if counter < 90.0 {
                            counter+=1.0;
                        }
                        else {
                            counter = counter;
                        }
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems..");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems..");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems...");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems...");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems....");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems....");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.....");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.....");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems......");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems......");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.......");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.......");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems........");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems........");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.........");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems.........");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems..........");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems..........");
                        thread::sleep(Duration::from_millis(500));

                        set_progress_bar(&mut progress_bar, counter, "Installing Systems...........");
                        thread::sleep(Duration::from_millis(500));
                        set_progress_bar(&mut progress_bar, counter, "Installing Systems...........");
                        thread::sleep(Duration::from_millis(500));
                    }
                }

                let clone_current_password = current_password.clone();
                let clone_rootuuid = rootuuid.clone();

                let (thread_message_sender, thread_message_reciever) = mpsc::channel::<&str>();

                set_progress_bar(&mut progress_bar,90.0, "Finishing Setup Processes");
                thread::spawn(move || {
                    thread_message_sender.send("finishing setup starting").unwrap();
                    let (_code, _output, _error) = linux::cleanup_mount_after_run(&clone_current_password, &clone_rootuuid);
                    thread_message_sender.send("finishing setup finished").unwrap();
                });

                counter = 90.0;

                loop {
                    if thread_message_reciever.try_recv().unwrap_or("") == "finishing setup finished"{
                        break;
                    }
                    else {
                        if counter < 99.0 {
                            counter+=1.0;
                        }
                        else {
                            counter = counter;
                        }
                    }
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes");
                    thread::sleep(Duration::from_millis(500));
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes");
                    thread::sleep(Duration::from_millis(500)); 

                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes.");
                    thread::sleep(Duration::from_millis(500));
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes.");
                    thread::sleep(Duration::from_millis(500)); 

                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes..");
                    thread::sleep(Duration::from_millis(500));
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes..");
                    thread::sleep(Duration::from_millis(500)); 

                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes...");
                    thread::sleep(Duration::from_millis(500));
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes...");
                    thread::sleep(Duration::from_millis(500)); 

                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes....");
                    thread::sleep(Duration::from_millis(500));
                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes....");
                    thread::sleep(Duration::from_millis(500)); 

                    set_progress_bar(&mut progress_bar, counter, "Finishing Setup Processes.....");
                    thread::sleep(Duration::from_millis(500)); 
                }
                set_progress_bar(&mut progress_bar,100.0, "Finished!");
            }
            if msg == "browse_go" {
                stop(&mut start_button, &mut browse_button, &mut choice_box);
                file_box = browse_file(&mut file_frame);
            }
        }
        start(&mut start_button, &mut browse_button, &mut choice_box);
    }
    app.run().unwrap();
}