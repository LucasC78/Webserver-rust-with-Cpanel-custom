<script>
    async function deleteFolder(folderName) {
        if (!confirm(`Are you sure you want to delete the folder: ${folderName}?`)) {
            return;
        }

        try {
            const response = await fetch('/delete_folder', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ folder_name: folderName }),
            });

            if (response.ok) {
                alert("Folder deleted successfully!");
                location.reload(); // Recharger la page
            } else {
                const errorText = await response.text();
                alert(`Error deleting folder: ${errorText}`);
            }
        } catch (error) {
            console.error("Error:", error);
            alert("An unexpected error occurred.");
        }
    }
</script>
