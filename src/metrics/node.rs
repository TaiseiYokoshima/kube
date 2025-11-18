use super::querier::TopLevelMetric;


#[derive(Debug, Clone)]
pub struct NodeMetric
{
   pub uid: String,
   pub metric: TopLevelMetric,
}


#[derive(Debug, Default)]
pub struct NodeMetricCollector {
   prev_cpu: f64,
   prev_time: i64,
   cpu_percentages: Vec<f64>,
   timestamps: Vec<f64>,
}


impl NodeMetricCollector {
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

   pub fn interporlate(&self, time: f64) -> f64 {
      let mut prev = 0;

      for (index, current ) in self.timestamps.iter().enumerate() {
         if *current == time {
            return self.cpu_percentages[index];
         };
          
         if *current > time {
            let t1 = self.timestamps[prev];
            let v1 = self.cpu_percentages[prev];

            let t2 = self.timestamps[index];
            let v2 = self.cpu_percentages[index];

            if t1 == t2 {
               return v2;
            };

            let t = time;
            return interpolate(t, t1, v1, t2, v2)
         };
         
         prev = index;
      };

      return 0.0;
   }
}

  
 
fn interpolate(t: f64, mut t1: f64, mut v1: f64, t2: f64, v2: f64) -> f64 {
   if t1 == t2 {
      t1 = 0.0;
      v1 = 0.0;
   };

   v1 + (v2 - v1) * (t - t1) / (t2 - t1)
}
