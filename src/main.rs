use std::fs::File; // needed for opening and reading file
use std::io::prelude::*;
use std::io::BufReader;

use chrono::{Datelike, Local, NaiveDateTime, TimeZone}; // handle time stuff

use druid::widget::{Align, Button, CrossAxisAlignment, Flex, MainAxisAlignment, RawLabel, Scroll};
use druid::{
    commands, AppDelegate, AppLauncher, Command, DelegateCtx, Env, FileDialogOptions, FileSpec,
    FontDescriptor, FontFamily, Handled, LocalizedString, Target, Widget, WindowDesc,
};

struct Delegate; // used by druid

/// Struct that hold one Step-Fit data set
/// Consisting of date, start and end times and step count
struct StepCollection {
    date: i32,
    start_time: i32,
    end_time: i32,
    steps: i32,
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder) // build the druid main window
        .title(
            LocalizedString::new("Step-Fit Log-File Analyzer")
                .with_placeholder("Step-Fit Log-File Analyzer"),
        )
        .window_size((700.0, 800.0));
    // inital text of the main program label
    let data = "Dieses Programm benötigt die Datei \"step-synchronize.log\"\n\
                die vom Programm \"Schritte auslesen.jnlp\" erzeugt wird.\n\
                Bei den meisten befindet sie sich höchstwahrscheinlich\n\
                im \"Download\" Ordner.\n\n\
                Hier werden hoffentlich gleich deine täglichen Schritte stehen ;)."
        .to_owned();
    AppLauncher::with_window(main_window) // launch the app UI
        .delegate(Delegate)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<String> {
    // define the UI
    let log = FileSpec::new("Log File", &["log"]); // file type options for the file open dialog
    let txt = FileSpec::new("Text file", &["txt"]);
    let default_name = String::from("step-synchronizer.log"); // default file name for open dialog
    let open_dialog_options = FileDialogOptions::new() // create the file open dialog
        .allowed_types(vec![log, txt])
        .default_type(log)
        .default_name(default_name)
        .title("Bitte die Log-Datei auswählen")
        .name_label("Log-Datei")
        .title("Bitte hier die Step-Fit Log-Datei auswählen")
        .button_text("Importieren");

    let mut my_label = RawLabel::new(); // the label holding explanation and output text
    my_label.set_font(FontDescriptor::new(FontFamily::MONOSPACE)); // font for label
                                                                   // create the button
    let open = Button::new("Hier klicken und die Datei step-synchronize.log auswählen").on_click(
        move |ctx, _, _| {
            ctx.submit_command(Command::new(
                druid::commands::SHOW_OPEN_PANEL,
                open_dialog_options.clone(),
                Target::Auto,
            ))
        },
    );

    let mut col = Flex::column(); // place a column to place the UI elements

    col.add_child(open); // place the open button
    col.add_spacer(8.0); // place a spacer
    col.add_child(my_label); // place the label
    col.set_cross_axis_alignment(CrossAxisAlignment::Center); // define alignments
    col.set_main_axis_alignment(MainAxisAlignment::Center);
    let scrollable = Scroll::new(col); // place the column in a scrollable box
    Align::centered(scrollable) // align the scrollable box centered
}

impl AppDelegate<String> for Delegate {
    // function that is called on button press (?)
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut String,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            // get info about the selected file
            println!("{:?}", file_info);
            println!("{:?}", file_info.path());
            let mut output_string = String::new(); // initialize the string that will hold the results
                                                   // ToDo: fetch the result here and handle the error case correctly
            step_analyze(file_info, &mut output_string); // call file analyzation function
            *data = output_string.to_owned(); // put result string into the UI label

            return Handled::Yes; // done
        }
        Handled::No
    }
}

/// the function that performs the file analyzation
fn step_analyze(my_file: &druid::FileInfo, output_string: &mut String) -> std::io::Result<()> {
    println!("This is Step-Fit Log-file Analyzer"); // say hello

    let file = File::open(my_file.path())?; // open the file
    let mut buf_reader = BufReader::new(file); // initate the buffered reader
    let mut line = String::new(); // strint that hold the current line
    let mut len = 1; // the line length information
    let mut counter: i32 = 0; // the line counter

    let mut step_vec = Vec::<StepCollection>::new(); // initialize the vector that holds all step structs

    while len > 0 {
        // do as long as line holds data
        counter = counter + 1; // increment counter
        line.clear(); // clear the string
        len = buf_reader.read_line(&mut line)?; // read the next line
        let content_read_result = line.find("\"items\":["); // search for string part specific for data line
        if content_read_result.is_some() {
            // if found
            let content = content_read_result.unwrap(); // get position of the found string part
            println!("Found step data in line {}", counter); // report success
                                                             //            println!("{}", content);
                                                             //            println!("{}", line);
            line.drain(..content); // throw away stuff before the relevant stuff
                                   //            println!("{}", line);
            parse_data_line(&mut line, &mut step_vec); // get info from the line
        }
        //        else {
        //            println!("{}, Nope", counter);
        //        }
    }
    let by_days: Vec<StepCollection> = sort_by_days(&step_vec); // sort the result vector
    let mut wochentag; // the day of the week
    let mut total_steps = 0; // the statistics
    let mut total_days = 0;

    let mut partial_string;
    for i in &by_days {
        // for all step collections in the result vector
        match NaiveDateTime::from_timestamp(i.date.into(), 0).weekday() {
            //translate weekdays to german
            chrono::Weekday::Mon => wochentag = "Montag".to_string(),
            chrono::Weekday::Tue => wochentag = "Dienstag".to_string(),
            chrono::Weekday::Wed => wochentag = "Mittwoch".to_string(),
            chrono::Weekday::Thu => wochentag = "Donnerstag".to_string(),
            chrono::Weekday::Fri => wochentag = "Freitag".to_string(),
            chrono::Weekday::Sat => wochentag = "Samstag".to_string(),
            chrono::Weekday::Sun => wochentag = "Sonntag".to_string(),
        }

        partial_string = format!(
            // format the result string for the given dataset
            "Du bist am {:10} den {} zwischen {} Uhr und {} Uhr {:5} Schritte gegangen",
            wochentag,
            Local.timestamp(i.date.into(), 0).format("%d-%m-%Y"),
            NaiveDateTime::from_timestamp(i.start_time.into(), 0).format("%H:%M"),
            NaiveDateTime::from_timestamp(i.end_time.into(), 0).format("%H:%M"),
            i.steps
        );
        println!("{}", partial_string); // print it to console
        output_string.push_str(&partial_string); // add it to result string
        output_string.push_str("\n"); // new line to result string
        total_steps = total_steps + i.steps; // do stats
        total_days = total_days + 1;
    }
    partial_string = format!(
        // format the stats string
        "Das sind {} Schritte in {} Tagen, also im Schnitt {} Schritte pro Tag ",
        total_steps,
        total_days,
        total_steps / total_days
    );
    println!("{}", &partial_string); // print it
    output_string.push_str(&partial_string); // add it to result string
    Ok(())
}

/// function that fetches the data from a given result line
fn parse_data_line(line: &mut String, step_vec: &mut Vec<StepCollection>) {
    let mut pos_curly_bracket; // position of the char at the end of a dataset
    let mut date;
    let mut start_time;
    let mut end_time;
    let mut steps;

    let mut pos_next_item: usize = line.find("\"date\":").unwrap_or(0); // find the date
    while pos_next_item > 0 {
        // as long as data sets can be found do ...
        date = (&line[pos_next_item + 7..pos_next_item + 17]) // fetch the date
            .parse()
            .unwrap_or(-1);
        //        println!("{}", date);

        pos_next_item = line.find("\"startTime\":").unwrap_or(0); // find startTime next ...
        start_time = (&line[pos_next_item + 12..pos_next_item + 22]) // ... and fetch it
            .parse()
            .unwrap_or(-1);
        //        println!("{}", start_time);

        pos_next_item = line.find("\"endTime\":").unwrap_or(0); // now end time
        end_time = (&line[pos_next_item + 10..pos_next_item + 20]) // you know the drill
            .parse()
            .unwrap_or(-1);
        //        println!("{}", end_time);

        pos_next_item = line.find("\"steps\":").unwrap_or(0); // find the steps
        line.drain(..pos_next_item + 1); // clear everything we have parsed so far
        pos_curly_bracket = line.find("}").unwrap_or(0); // we do not know the number of digits for steps, so find end brackets position
        steps = (&line[7..pos_curly_bracket]).parse().unwrap_or(-1); // fetch the steps
                                                                     //        println!("{}", steps);

        pos_next_item = line.find("\"date\":").unwrap_or(0); // aaand find the start of the next dataset
                                                             //        println!("{}, Nope", pos_next_item);
                                                             //        println!("{}", line);
                                                             // ceck if this dataset was already found somewhere else (just in case)
        if check_for_duplicates(date, start_time, end_time, steps, &step_vec) == 1 {
            step_vec.push(StepCollection {
                // add the dataset to the result vector
                date,
                start_time,
                end_time,
                steps,
            });
        }
    }
}

/// function that checks for duplicate datasets
fn check_for_duplicates(
    date: i32,
    start_time: i32,
    end_time: i32,
    steps: i32,
    step_vec: &Vec<StepCollection>,
) -> i32 {
    for i in step_vec {
        // if everything is the same ...
        if i.date == date
            && i.start_time == start_time
            && i.end_time == end_time
            && i.steps == steps
        {
            return 0; // say so
        }
    }
    return 1; // if not, say otherwise
}

/// function that combines all the records from one date.
/// sorts the result-vector by date
fn sort_by_days(step_vec: &Vec<StepCollection>) -> Vec<StepCollection> {
    let mut day_already_recorded = 0; // flag for known date
    let mut days_vec = Vec::<StepCollection>::new(); // the sorted vector
    for i in step_vec {
        // for all records in the input vector
        for mut j in &mut days_vec {
            // for all records in the output vector
            if i.date == j.date {
                // if the current record has a date that is already known ...
                day_already_recorded = 1; // set the flag
                j.steps = j.steps + i.steps; // add its data to that record in the output vector
                if i.start_time < j.start_time {
                    j.start_time = i.start_time;
                }
                if i.end_time > j.end_time {
                    j.end_time = i.end_time;
                }
            }
        }
        if day_already_recorded == 0 {
            // if the date is not known yet ...
            days_vec.push(StepCollection {
                // ... create record in output vector
                date: i.date,
                start_time: i.start_time,
                end_time: i.end_time,
                steps: i.steps,
            });
        }
        day_already_recorded = 0; // reset the flag
    }
    days_vec.sort_by(|a, b| a.date.cmp(&b.date)); // sort the ouput vector by date
    days_vec // return it
}
