/*
 * file: main.rs
 * author: Kostiantyn Konakhevych
 * date: 2026-03-13

* ● Chapter 13 Task: Student Grade Analyzer
* Create a program that processes student exam data using iterators and closures throughout.
* No manual for loops allowed — everything must be done with iterator chains.

* Requirements

* Create a file ch13_iterators/src/main.rs in a new cargo project:

* 1. Define the data:

* struct Student {
*     name: String,
*     scores: Vec<u32>, // exam scores 0-100
* }

* 2. Implement Iterator for a custom struct TopStudents that wraps a sorted list of (String, f64) (name, average)
* and yields students with average >= 70.0, one at a time.
* 3. Using only iterator chains, implement these functions:

* - averages(students: &[Student]) -> Vec<(String, f64)> — returns each student's name and average score,
*   sorted by average descending
* - highest_single_score(students: &[Student]) -> Option<(String, u32)> — returns the name and value of the highest individual score across all students (use flat_map)
* - grade_distribution(students: &[Student]) -> Vec<(char, usize)> — counts how many individual scores fall into each grade:
*       A (90-100), B (80-89), C (70-79), D (60-69), F (0-59). Return as [('A', count), ('B', count), ...]
* - top_students(students: &[Student]) -> TopStudents — returns your custom iterator

* 4. In main(), create sample data (at least 5 students, 3+ scores each) and demonstrate all functions. Use inspect somewhere in a chain for debug output.

* Focus on: closures capturing environment, map/filter/flat_map/fold/enumerate/zip, consuming adaptors, and your custom iterator.
*/

use std::collections::HashMap;

const MIN_TOP_AVERAGE: f64 = 70.0;
const MAX_SCORE: u32 = 100;

enum Grade {
    A,
    B,
    C,
    D,
    F,
}

impl Grade {
    // For simplifing I didn't implement error procesing, if score > MAX,
    // but possible it better to manager on input check side, which is not part of this project
    fn get_grade(score: u32) -> Grade {
        match score {
            0..=59 => Grade::F,
            60..=69 => Grade::D,
            70..=79 => Grade::C,
            80..=89 => Grade::B,
            90..=MAX_SCORE => Grade::A,
            _ => Grade::F,
        }
    }
}

//impl From<u32> for Grade {
//    fn from(value: u32) -> Self {
//        Grade::get_grade(value)
//    }
//}

impl From<Grade> for char {
    fn from(value: Grade) -> char {
        match value {
            Grade::A => 'A',
            Grade::B => 'B',
            Grade::C => 'C',
            Grade::D => 'D',
            Grade::F => 'F',
        }
    }
}

struct Student {
    name: String,
    scores: Vec<u32>, // exam scores 0-100
}

impl Student {
    fn new(name: String, scores: Vec<u32>) -> Self {
        Student { name, scores }
    }

    fn average(&self) -> Option<f64> {
        let sum: f64 =
            <u32 as Into<f64>>::into(self.scores.iter().sum::<u32>()) / self.scores.len() as f64;
        if sum.is_nan() { None } else { Some(sum) }
    }

    // fn highest_score(&self) -> Option<u32> {
    //     self.scores.iter().max().cloned()
    // }
    //
    // fn add_score(&mut self, score: u32) -> Result<bool, &str> {
    //     if score <= MAX_SCORE {
    //         self.scores.push(score);
    //         Ok(true)
    //     } else {
    //         Err("Score value is out of scope!")
    //     }
    // }

    fn grades(&self) -> Vec<Grade> {
        self.scores.iter().map(|s| Grade::get_grade(*s)).collect()
    }
}

struct TopStudents {
    students: std::vec::IntoIter<(String, f64)>,
}

impl Iterator for TopStudents {
    type Item = (String, f64);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(s) = self.students.next() {
            if s.1 >= MIN_TOP_AVERAGE {
                return Some(s);
            }
        }
        None
    }
}

fn averages(students: &[Student]) -> Vec<(String, f64)> {
    // made this one for task
    let names: Vec<String> = students.iter().map(|s| s.name.clone()).collect();
    let mut avg: Vec<(String, f64)> = names
        .into_iter()
        .zip(
            students
                .iter()
                .map(|s| s.average().unwrap_or(0.0))
                .collect::<Vec<f64>>(),
        )
        .collect();

    // let mut avg: Vec<(String, f64)> = students
    //     .iter()
    //     .map(|s| (s.name.clone(), s.average()))
    //     .collect();

    avg.sort_by(|a, b| b.1.total_cmp(&a.1));
    avg
}

fn highest_single_score(students: &[Student]) -> Option<(String, u32)> {
    //students
    //    .iter()
    //    .map(|s| (s.name.clone(), s.highest_score().unwrap_or(0)))
    //    .max_by(|s1, s2| s1.1.cmp(&s2.1))

    // Another variant
    students
        .iter()
        .flat_map(|s| s.scores.iter().map(move |&score| (s.name.clone(), score)))
        .inspect(|x| println!("after flat_map: {x:?}"))
        .max_by_key(|(_, score)| *score)

    // don't use clone
    //  students.iter()
    //  .flat_map(|s| s.scores.iter().map(move |score| (&s.name, score)))
    //  .max_by_key(|(_, &score)| score)
    //  .map(|(name, &score)| (name.clone(), score)
}

fn grade_distribution(students: &[Student]) -> Vec<(char, usize)> {
    let mut dist: Vec<(char, usize)> = students
        .iter()
        .flat_map(|s| s.grades().into_iter().map(|g| g.into()))
        .fold(HashMap::new(), |mut map, grade| {
            *map.entry(grade).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect();

    dist.sort_by_key(|(g, _)| *g);
    dist
}

/// Return iterator of TopStudents of Students slice
///
/// # Arguments
///
/// * `argument_name` - type and description.
///
/// # Returns
/// type and description of the returned object.
///
/// # Examples
/// ```rust
/// write me later
/// ```
fn top_students(students: &[Student]) -> TopStudents {
    let average = averages(students);

    TopStudents {
        students: average.into_iter(),
    }
}

fn main() {
    let students = vec![
        Student::new("Boris".to_string(), vec![70, 50, 60]),
        Student::new("Kolya".to_string(), vec![90, 80, 75]),
        Student::new("Sasha".to_string(), vec![45, 85, 90]),
        Student::new("Kiva".to_string(), vec![30, 65, 45]),
        Student::new("Rostislav".to_string(), vec![93, 65, 66]),
    ];

    println!("Grade distribution: {:#?}", grade_distribution(&students));
    println!(
        "Highest single score: {:#?}",
        highest_single_score(&students)
    );
    println!("Averages: {:#?}", averages(&students));

    let (names, avg_scrores): (Vec<String>, Vec<f64>) = top_students(&students).unzip();
    println!(" {:<15} {:>6}", "Name", "Avg");
    println!("{:-<15} {:->6}", "", "");
    names
        .into_iter()
        .enumerate()
        .for_each(|(i, n)| println!("{}. {n:<11} {:>6.2}", i + 1, avg_scrores[i]));
}
