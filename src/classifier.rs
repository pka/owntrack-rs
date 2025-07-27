use smartcore::ensemble::random_forest_classifier::RandomForestClassifier;
use smartcore::linalg::basic::matrix::DenseMatrix;

pub struct Classifier {
    tree: RandomForestClassifier<f64, u16, DenseMatrix<f64>, Vec<u16>>,
}

static MODEL_JSON: &str = include_str!("model.json");

impl Classifier {
    pub fn load() -> Self {
        let tree = serde_json::from_str(MODEL_JSON).unwrap();
        Self { tree }
    }
    pub fn predict(&self, x: &Vec<Vec<f64>>) -> Vec<u16> {
        let matrix = DenseMatrix::from_2d_vec(x).unwrap();
        self.tree.predict(&matrix).unwrap()
    }
}
