use dtracker::tracker::bt_tracker::{BtTracker, BtTrackerError};

fn main() -> Result<(), BtTrackerError> {
    BtTracker::init()?.run()?;

    Ok(())
}
