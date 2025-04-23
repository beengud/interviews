import { useEffect, useState, ChangeEvent } from "react";
import {
  DataGrid,
  GridColDef,
  GridRenderCellParams,
} from "@mui/x-data-grid";
import {
  Container,
  Typography,
  Button,
  Box,
  TextField,
  Stack,
} from "@mui/material";
import axios from "axios";

const API_BASE = import.meta.env.VITE_API_BASE_URL;

type Artifact = {
  id: string;
  original_filename: string;
  size: number;
  sha256: string;
  upload_time: string;
  metadata: string;
};

function App() {
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [file, setFile] = useState<File | null>(null);

  const fetchArtifacts = async () => {
    try {
      const res = await axios.get<Artifact[]>(`${API_BASE}/artifacts`);
      setArtifacts(res.data);
    } catch (err) {
      console.error("Failed to fetch artifacts", err);
    }
  };

  const upload = async () => {
    if (!file) return;
    const formData = new FormData();
    formData.append("file", file);

    try {
      await axios.post(`${API_BASE}/upload`, formData, {
        headers: { "Content-Type": "multipart/form-data" },
      });
      setFile(null);
      fetchArtifacts();
    } catch (err) {
      console.error("Upload failed", err);
    }
  };

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    setFile(e.target.files?.[0] || null);
  };

  const download = (id: string) => {
    window.location.href = `${API_BASE}/download/${id}`;
  };

  const columns: GridColDef[] = [
    { field: "original_filename", headerName: "Filename", flex: 2, minWidth: 180 },
    { field: "metadata", headerName: "Metadata", flex: 2, minWidth: 150 },
    { field: "size", headerName: "Size (bytes)", type: "number", minWidth: 100 },
    { field: "upload_time", headerName: "Uploaded At", flex: 1, minWidth: 200 },
    { field: "sha256", headerName: "SHA256", flex: 3, minWidth: 300 },
    {
      field: "download",
      headerName: "Actions",
      sortable: false,
      minWidth: 130,
      renderCell: (params: GridRenderCellParams<Artifact>) => (
        <Button
          variant="contained"
          size="small"
          fullWidth
          onClick={() => download(params.row.id)}
        >
          Download
        </Button>
      ),
    },
  ];

  useEffect(() => {
    fetchArtifacts();
  }, []);

  return (
    <Container maxWidth="lg" sx={{ mt: 4, pb: 4 }}>
      <Typography variant="h4" gutterBottom align="center">
        Secure Artifact Manager
      </Typography>

      <Stack direction="row" spacing={2} alignItems="center" justifyContent="center" mb={3}>
        <TextField
          type="file"
          onChange={handleFileChange}
        />
        <Button variant="contained" onClick={upload} disabled={!file}>
          Upload
        </Button>
      </Stack>

      <Box sx={{ height: 500, width: "100%", overflowX: "auto", backgroundColor: "#fff", borderRadius: 2, boxShadow: 1 }}>
        <DataGrid
          rows={artifacts}
          getRowId={(row: Artifact) => row.id}
          columns={columns}
          pageSizeOptions={[5, 10]}
          initialState={{
            pagination: {
              paginationModel: { pageSize: 5, page: 0 },
            },
          }}
        />
      </Box>
    </Container>
  );
}

export default App;