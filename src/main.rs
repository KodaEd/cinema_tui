

mod app;
use app::App;

fn main() -> () {
    let mut app = App::new();
    app.get_ritz_movies();

    println!("{:?}", app.ritz_movie_times);
}