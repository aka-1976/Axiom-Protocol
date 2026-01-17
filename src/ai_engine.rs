

pub struct NeuralGuardian {
    weights: [f32; 3],
    learning_rate: f32,
}

impl NeuralGuardian {
    pub fn new() -> Self {
        Self {
            weights: [0.5, 0.3, 0.2],
            learning_rate: 0.01,
        }
    }

    pub fn predict_trust(&self, time_delta: f32, consistency: f32, depth: f32) -> bool {
        let score = (time_delta * self.weights[0]) + 
                    (consistency * self.weights[1]) + 
                    (depth * self.weights[2]);
        score > 0.4
    }

    pub fn train(&mut self, inputs: [f32; 3], target: f32) {
        for i in 0..3 {
            let prediction = (inputs[0] * self.weights[0]) + (inputs[1] * self.weights[1]) + (inputs[2] * self.weights[2]);
            let error = target - prediction;
            self.weights[i] += self.learning_rate * error * inputs[i];
        }
    }
}

impl Default for NeuralGuardian {
    fn default() -> Self {
        Self::new()
    }
}
