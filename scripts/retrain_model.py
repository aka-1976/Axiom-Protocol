import pandas as pd
import numpy as np
from sklearn.neural_network import MLPClassifier
import onnx
from skl2onnx import convert_sklearn
from skl2onnx.common.data_types import FloatTensorType

# Load collected data
data = pd.read_csv('../training_data.csv', 
                   names=['msg_rate', 'history', 'reputation', 'label'])

X = data[['msg_rate', 'history', 'reputation']].values
y = data['label'].values

# Train improved model
model = MLPClassifier(hidden_layer_sizes=(10, 8), max_iter=1000)
model.fit(X, y)

print(f"Model accuracy: {model.score(X, y):.2%}")

# Convert to ONNX
initial_type = [('float_input', FloatTensorType([None, 3]))]
onnx_model = convert_sklearn(model, initial_types=initial_type)

# Save new model
with open('../guardian.onnx', 'wb') as f:
    f.write(onnx_model.SerializeToString())

print("✅ Model retrained and saved to guardian.onnx")
