#[derive(Debug, Default)]
pub struct TotalMetric {
   prev_cpu: f64,
   prev_time: i64,
   cpu_percentages: Vec<f64>,
   timestamps: Vec<f64>,
}


impl TotalMetric {
   pub fn new() -> Self {
      Self::default()
   }

   pub fn next(&mut self, time: i64, cpu: f64) -> Option<(f64, f64)> {
      let time_d = time - self.prev_time;

      if time_d == 0 {
         let timestamp = self.timestamps.get(self.timestamps.len() - 1);
         let cpu = self.cpu_percentages.get(self.cpu_percentages.len() - 1);
         match (timestamp, cpu) {
            (Some(t), Some(c)) => return Some((*t, *c)),
            _ => return None,
         };
      };

      let cpu_d = cpu - self.prev_cpu;
      assert!(time_d.is_positive());
      assert!(cpu_d.is_sign_positive());

      let percentage = (cpu_d / (time_d as f64 / 1000.0)) * 100.0;
      let timestamp = (time + self.prev_time) as f64 / 2.0;
      self.timestamps.push(timestamp);
      self.cpu_percentages.push(percentage);


      self.prev_cpu = cpu;
      self.prev_time = time;
      Some((timestamp, percentage))
   }
}

