<template>
    <div class="tourHeader">
        <h3>
            Tour Selection
        </h3>
        <br/>
        <img src="../assets/paper-plane.svg" width="128" height="128">
        <div class="tourVersion">
            <label>Tour Booking Version</label>
            <select v-model="selectedVersion" class="versionSelect">
                <option value="version1">Version 1</option>
                <option value="version2">Version 2</option>
            </select>
        </div>
    </div>
    <form @submit.prevent="handleSubmit">
        <label>Name:</label>
        <input type="text" required v-model="name">

        <label>Email:</label>
        <input type="email" required v-model="email">
        <div v-if="emailError" class="error">
            {{ emailError }}
        </div>

        <div class="book">
            <label>Book: </label>
            <input type="checkbox" v-model="book" @change="handleExclusive('book')">
            <label>Cancel: </label>
            <input type="checkbox" v-model="cancel" @change="handleExclusive('cancel')">
        </div>

        <label>Tours: </label>
        <select v-model="tour" class="tourSelect">
            <option value="copenhagen">Copenhagen</option>
            <option value="wien">Wien</option>
            <option value="krakow">Krakow</option>
            <option value="berlin">Berlin</option>
            <option value="london">London</option>
        </select>

        <div v-if="selectedVersion === 'version2'">
            <label>Class:</label>
            <select v-model="selectedClass" class="classSelect">
                <option value="economic">Economic</option>
                <option value="business">Business</option>
                <option value="first">First</option>
            </select>
        </div>

        <div class="submit">
            <button>Submit</button>
        </div>
    </form>
 </template>

<script>
    import axios from 'axios'

    export default {
        data() {
            return {
                email: '',
                name: '',
                book: false,
                cancel: false,
                tour: '',
                emailError: '',
                selectedVersion: 'version1',
                selectedClass: ''
            }
        },
        methods: {
            handleExclusive(box) {
                if (box === 'book' && this.book) {
                    this.cancel = false;
                } else if (box === 'cancel' && this.cancel) {
                    this.book = false;
                }
            },
            handleSubmit(e) {
                this.emailError = this.email.length > 5 ? 
                    '' : 'There`s no way your email is that short';
                if (!this.emailError) {
                    if (this.selectedVersion === 'version1') {
                        const url = 'http://localhost:8000/book';
                        const data = {
                          book: this.book,
                          cancel: this.cancel,
                          name: this.name,
                          email: this.email,
                          location: this.tour
                        };
    
                        fetch(url, {
                          method: 'POST',
                          headers: {
                            'Content-Type': 'application/json'
                          },
                          body: JSON.stringify(data)
                        })
                          .then(response => {
                            if (!response.ok) {
                              throw new Error('Network response was not ok');
                            }
                            return response.json();
                          })
                          .then(data => {
                            console.log(data);
                          })
                          .catch(error => {
                            console.error('There was a problem with the fetch operation:', error);
                          });
                    } else {
                        const url = 'http://localhost:8000/bookv2';
                        const data = {
                            booking: {
                                book: this.book,
                                cancel: this.cancel,
                                name: this.name,
                                email: this.email,
                                location: this.tour
                            },
                            class: this.selectedClass
                        };
    
                        fetch(url, {
                          method: 'POST',
                          headers: {
                            'Content-Type': 'application/json'
                          },
                          body: JSON.stringify(data)
                        })
                          .then(response => {
                            if (!response.ok) {
                              throw new Error('Network response was not ok');
                            }
                            return response.json();
                          })
                          .then(data => {
                            console.log(data);
                          })
                          .catch(error => {
                            console.error('There was a problem with the fetch operation:', error);
                          });
                    }
                    alert('Form submitted!');
                }
            }
        }
    }
</script>

<style>
    form {
        max-width: 420px;
        margin: 30px auto;
        background: white;
        text-align: left;
        padding: 40px;
        border-radius: 10px;
    }
    label {
        color: #aaa;
        display: inline-block;
        margin: 25px 0 15px;
        font-size: 0.6em;
        text-transform: uppercase;
        letter-spacing: 1px;
        font-weight: bold;
    }
    input, select {
        display: block;
        padding: 10px 6px;
        box-sizing: border-box;
        border: none;
        border-bottom: 1px solid #ddd;
        color: #555;
    }
    input[type="checkbox"] {
        display: inline-block;
        width: 16px;
        margin: 0 10px 0 0;
        position: relative;
        top: 2px;
        left: 4px;
    }
    h3 {
        color: #555;
        display: inline-block;
        margin: 50px 0 30px;
        font-size: 1em;
        text-transform: uppercase;
        letter-spacing: 2px;
        font-weight: bold;
    }
    button {
        background: #0b6dff;
        border: 0;
        padding: 10px 20px;
        margin-top: 20px;
        color: white;
        border-radius: 20px;
    }
    .submit {
        text-align: center;
    }
    .error {
        color: #ff0062;
        margin-top: 10px;
        font-size: 0.8em;
        font-weight: bold;
    }
    .tourHeader {
        display: inline-block;
    }
    .tourSelect, .classSelect {
        width: 250px;
    }
    .versionSelect {
        width: 150px;
    }
</style>