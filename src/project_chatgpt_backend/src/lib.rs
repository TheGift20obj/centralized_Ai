use candid::CandidType;
use serde::{Deserialize, Serialize};
use ic_cdk_macros::{init, query};
use std::cell::RefCell;
use ic_cdk::api::time;

thread_local! {
    // RNG state seeded from current time
    static RNG_STATE: RefCell<u64> = RefCell::new(time());
}

/// Simple xorshift64* RNG for pseudo-random numbers
fn pseudo_random() -> u64 {
    RNG_STATE.with(|state| {
        let mut s = *state.borrow();
        // xorshift64* steps
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        *state.borrow_mut() = s;
        s
    })
}

/// Get pseudo-random float in range [-1.0, 1.0]
fn pseudo_random_float() -> f32 {
    let rnd = pseudo_random();
    let val = (rnd as f64 / u64::MAX as f64) * 2.0 - 1.0;
    val as f32
}

/// Shuffle slice in-place using our RNG
fn shuffle_dataset<T>(dataset: &mut [T]) {
    let len = dataset.len();
    for i in (1..len).rev() {
        let j = (pseudo_random() as usize) % (i + 1);
        dataset.swap(i, j);
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct Neuron {
    pub weights: Vec<f32>,
    pub bias: f32,
    pub last_output: f32,
}

impl Neuron {
    pub fn activate(&mut self, inputs: &[f32]) -> f32 {
        let sum: f32 = self
            .weights
            .iter()
            .zip(inputs.iter())
            .map(|(w, i)| w * i)
            .sum();
        self.last_output = Self::sigmoid(sum + self.bias);
        self.last_output
    }

    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }

    fn sigmoid_derivative(output: f32) -> f32 {
        output * (1.0 - output)
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct Layer {
    pub neurons: Vec<Neuron>,
}

impl Layer {
    pub fn forward(&mut self, inputs: &[f32]) -> Vec<f32> {
        self.neurons.iter_mut().map(|n| n.activate(inputs)).collect()
    }
}

#[derive(Clone, CandidType, Serialize, Deserialize, Debug)]
pub struct NeuralNetwork {
    pub layers: Vec<Layer>,
}

impl NeuralNetwork {
    pub fn new(layer_sizes: &[usize]) -> Self {
        let mut layers = Vec::new();

        for w in layer_sizes.windows(2) {
            let input_size = w[0];
            let output_size = w[1];

            let neurons = (0..output_size)
                .map(|_| {
                    let weights = (0..input_size)
                        .map(|_| pseudo_random_float())
                        .collect();
                    let bias = pseudo_random_float();
                    Neuron {
                        weights,
                        bias,
                        last_output: 0.0,
                    }
                })
                .collect();

            layers.push(Layer { neurons });
        }

        Self { layers }
    }

    pub fn predict(&mut self, input: Vec<f32>) -> Vec<f32> {
        self.layers.iter_mut().fold(input, |inp, layer| layer.forward(&inp))
    }

    pub fn train_on_example(&mut self, input: &[f32], target: &[f32]) {
        if self.layers.is_empty() {
            return;
        }

        // Forward pass
        let mut activations = Vec::new();
        let mut current_input = input.to_vec();
        activations.push(current_input.clone());

        for layer in self.layers.iter_mut() {
            current_input = layer.forward(&current_input);
            activations.push(current_input.clone());
        }

        // Backward pass (backpropagation)
        let mut errors = Vec::new();
        let output_activations = activations.last().unwrap();
        errors.push(
            output_activations
                .iter()
                .zip(target.iter())
                .map(|(o, t)| t - o)
                .collect::<Vec<f32>>(),
        );

        // Calculate errors for hidden layers
        for l in (1..self.layers.len()).rev() {
            let layer = &self.layers[l];
            let prev_layer = &self.layers[l - 1];
            let mut layer_errors = vec![0.0; prev_layer.neurons.len()];

            for (i, neuron) in prev_layer.neurons.iter().enumerate() {
                let mut error_sum = 0.0;
                for (j, next_neuron) in layer.neurons.iter().enumerate() {
                    error_sum += errors[0][j] * next_neuron.weights[i];
                }
                layer_errors[i] = error_sum;
            }
            errors.insert(0, layer_errors);
        }

        // Update weights and biases
        let lr = 0.01;
        for (l, layer) in self.layers.iter_mut().enumerate() {
            let inputs = &activations[l];
            for (n_idx, neuron) in layer.neurons.iter_mut().enumerate() {
                let output = neuron.last_output;
                let delta = errors[l][n_idx] * Neuron::sigmoid_derivative(output);
                for (w_idx, w) in neuron.weights.iter_mut().enumerate() {
                    *w += lr * delta * inputs[w_idx];
                }
                neuron.bias += lr * delta;
            }
        }
    }

    pub fn train_on_dataset(&mut self, dataset: &mut [(Vec<f32>, Vec<f32>)], epochs: usize) {
        for _ in 0..epochs {
            shuffle_dataset(dataset);
            for (input, target) in dataset.iter() {
                self.train_on_example(input, target);
            }
        }
    }
}

// Example dialogs dataset
static EXAMPLE_DIALOGS: &[(&str, &str)] = &[
    ("Cześć", "Hej, jak mogę pomóc?"),
    ("Jaki jest kolor nieba?", "Niebieski"),
    ("Jak się masz?", "Dobrze, dziękuję!"),
    ("Opowiedz mi o kwiatkach", "Kwiatki są piękne i różnorodne."),
    ("Jaki jest twój nastrój?", "Czuję się świetnie!"),
];

// Encode string to input vector (normalized chars, max length 10)
fn encode_input(s: &str) -> Vec<f32> {
    let mut input: Vec<f32> = s
        .chars()
        .take(10)
        .map(|c| (c as u8 as f32) / 255.0)
        .collect();
    while input.len() < 10 {
        input.push(0.0);
    }
    input
}

// Encode string to target vector (normalized chars, max length 83)
fn encode_target(s: &str) -> Vec<f32> {
    let mut target: Vec<f32> = s
        .chars()
        .take(83)
        .map(|c| (c as u8 as f32) / 255.0)
        .collect();
    while target.len() < 83 {
        target.push(0.0);
    }
    target
}

thread_local! {
    static AI_BRAIN: RefCell<NeuralNetwork> = RefCell::new(NeuralNetwork::new(&[10, 750, 2500, 5000, 1250, 83]));
}

#[init]
fn init() {
    AI_BRAIN.with(|brain| {
        *brain.borrow_mut() = NeuralNetwork::new(&[10, 750, 2500, 5000, 1250, 83]);

        let mut net = brain.borrow_mut();

        let mut dataset: Vec<(Vec<f32>, Vec<f32>)> = EXAMPLE_DIALOGS
            .iter()
            .map(|(inp, tgt)| (encode_input(inp), encode_target(tgt)))
            .collect();

        net.train_on_dataset(&mut dataset, 10);
    });
}

#[query]
fn chat(message: String) -> String {
    let input = encode_input(&message);

    let output = AI_BRAIN.with(|brain| {
        let mut net = brain.borrow_mut();

        // Predict
        let prediction = net.predict(input.clone());

        // Optional: online training to adapt to new input slightly
        let target: Vec<f32> = prediction.iter().map(|v| (*v + 0.1).min(1.0)).collect();
        net.train_on_example(&input, &target);

        prediction
    });

    // Predefined responses
    let responses = [
        "Cześć", "Hej", "Witaj", "Dzień dobry", "Witam", "Halo", "Siema", "Czołem", "Hejka", "Yo",
        "Serdecznie witam", "Miło cię widzieć", "Czołem przyjacielu", "No hej", "Witaj ponownie",
        "Elo", "Siemka", "Pozdrowienia", "Cześć, jak mogę pomóc?", "W czym mogę pomóc?",
        "Jaki jest problem?", "Co chcesz wiedzieć?", "Zapraszam do rozmowy",
    ];

    let mut best_response = "";
    let mut best_score = 0f32;

    for resp in responses.iter() {
        let encoded_resp = encode_target(resp);
        let score: f32 = encoded_resp
            .iter()
            .zip(output.iter())
            .map(|(a, b)| 1.0 - (a - b).abs())
            .sum();

        if score > best_score {
            best_score = score;
            best_response = resp;
        }
    }

    best_response.to_string()
}
